use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use crate::repositories::{CreateTodo, TodoRepository};

pub async fn create_todo<T: TodoRepository>(
    State(repository): State<T>,
    Json(payload): Json<CreateTodo>,
) -> impl IntoResponse {
    let todo = repository.create(payload);

    (StatusCode::CREATED, Json(todo))
}