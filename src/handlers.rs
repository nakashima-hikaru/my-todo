use std::sync::Arc;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum::extract::Path;
use crate::repositories::{CreateTodo, TodoRepository, UpdateTodo};

pub async fn create_todo<T: TodoRepository>(
    State(repository): State<Arc<T>>,
    Json(payload): Json<CreateTodo>,
) -> impl IntoResponse {
    let todo = repository.create(payload);

    (StatusCode::CREATED, Json(todo))
}

pub async fn update_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    State(repository): State<Arc<T>>,
    Json(payload): Json<UpdateTodo>,
) -> Result<impl IntoResponse, StatusCode> {
    let todo = repository
        .update(id, payload)
        .or(Err(StatusCode::NOT_FOUND))?;
    Ok((StatusCode::CREATED, Json(todo)))
}
