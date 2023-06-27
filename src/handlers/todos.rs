use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::handlers::ValidatedJson;
use crate::repositories::todos::{CreateTodo, TodoRepository, UpdateTodo};

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
