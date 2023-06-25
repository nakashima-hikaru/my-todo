use std::sync::Arc;

use axum::body::HttpBody;
use axum::extract::{FromRequest, Path};
use axum::http::Request;
use axum::{async_trait, extract::State, http::StatusCode, response::IntoResponse, BoxError, Json};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::repositories::{CreateTodo, TodoRepository, UpdateTodo};

#[derive(Debug)]
pub(crate) struct ValidatedJson<T>(T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, &state)
            .await
            .map_err(|rejection| {
                let message = format!("Json parse error: [{}]", rejection);
                (StatusCode::BAD_REQUEST, message)
            })?;
        value.validate().map_err(|rejection| {
            let message = format!("Validation error: [{}]", rejection).replace('\n', ", ");
            (StatusCode::BAD_REQUEST, message)
        })?;
        Ok(ValidatedJson(value))
    }
}

pub(crate) async fn create_todo<T: TodoRepository>(
    State(repository): State<Arc<T>>,
    ValidatedJson(payload): ValidatedJson<CreateTodo>,
) -> anyhow::Result<(StatusCode, impl IntoResponse), StatusCode> {
    let todo = repository
        .create(payload)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok((StatusCode::CREATED, Json(todo)))
}

pub(crate) async fn update_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    State(repository): State<Arc<T>>,
    ValidatedJson(payload): ValidatedJson<UpdateTodo>,
) -> Result<(StatusCode, impl IntoResponse), StatusCode> {
    let todo = repository
        .update(id, payload)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok((StatusCode::CREATED, Json(todo)))
}

pub(crate) async fn all_todo<T: TodoRepository>(
    State(repository): State<Arc<T>>,
) -> anyhow::Result<(StatusCode, impl IntoResponse), StatusCode> {
    let todo = repository.all().await.unwrap();
    Ok((StatusCode::OK, Json(todo)))
}

pub(crate) async fn find_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    State(repository): State<Arc<T>>,
) -> Result<(StatusCode, impl IntoResponse), StatusCode> {
    let todo = repository
        .find(id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok((StatusCode::OK, Json(todo)))
}

pub(crate) async fn delete_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    State(repository): State<Arc<T>>,
) -> StatusCode {
    if repository.delete(id).await.is_ok() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}
