use std::sync::Arc;

use axum::{async_trait, BoxError, extract::State, http::StatusCode, Json, response::IntoResponse};
use axum::body::HttpBody;
use axum::extract::{FromRequest, Path};
use axum::http::Request;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::repositories::{CreateTodo, TodoRepository, UpdateTodo};

// todo: refine error propagation (error messages in `from_request` seems to be vanished)
#[derive(Debug)]
pub struct ValidatedJson<T>(T);

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
        let Json(value) = Json::<T>::from_request(req, &state).await.map_err(|rejection| {
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

pub async fn create_todo<T: TodoRepository>(
    State(repository): State<Arc<T>>,
    ValidatedJson(payload): ValidatedJson<CreateTodo>,
) -> (StatusCode, impl IntoResponse) {
    let todo = repository.create(payload);

    (StatusCode::CREATED, Json(todo))
}

pub async fn update_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    State(repository): State<Arc<T>>,
    ValidatedJson(payload): ValidatedJson<UpdateTodo>,
) -> Result<(StatusCode, impl IntoResponse), StatusCode> {
    let todo = repository
        .update(id, payload)
        .or(Err(StatusCode::NOT_FOUND))?;
    Ok((StatusCode::CREATED, Json(todo)))
}

pub async fn all_todo<T: TodoRepository>(
    State(repository): State<Arc<T>>,
) -> (StatusCode, impl IntoResponse) {
    let todo = repository.all();
    (StatusCode::OK, Json(todo))
}

pub async fn find_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    State(repository): State<Arc<T>>,
) -> Result<(StatusCode, impl IntoResponse), StatusCode> {
    let todo = repository.find(id).ok_or(StatusCode::NOT_FOUND)?;
    Ok((StatusCode::OK, Json(todo)))
}

pub async fn delete_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    State(repository): State<Arc<T>>,
) -> StatusCode {
    if repository.delete(id).is_ok() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}
