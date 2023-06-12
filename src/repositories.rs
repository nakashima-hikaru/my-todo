use axum::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use thiserror::Error;
use validator::Validate;

pub mod hash_map_repository;
pub mod database_repository;

#[derive(Debug, Error)]
enum RepositoryError {
    #[error("Unexpected error: [{0}]")]
    Unexpected(String),
    #[error("NotFound, id: {0}")]
    NotFound(i32),
}

#[async_trait]
pub trait TodoRepository: Clone + Send + Sync + 'static {
    async fn create(&self, payload: CreateTodo) -> anyhow::Result<Todo>;
    async fn find(&self, id: i32) -> anyhow::Result<Todo>;
    async fn all(&self) -> anyhow::Result<Vec<Todo>>;
    async fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo>;
    async fn delete(&self, id: i32) -> anyhow::Result<()>;
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, FromRow)]
pub struct Todo {
    pub id: i32,
    pub text: String,
    pub completed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Validate)]
pub struct CreateTodo {
    #[validate(length(min = 1, message = "text must not be empty"))]
    #[validate(length(max = 100, message = "text length exceeds the limit"))]
    text: String,
}

#[cfg(test)]
impl CreateTodo {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Validate)]
pub struct UpdateTodo {
    #[validate(length(min = 1, message = "text must not be empty"))]
    #[validate(length(max = 100, message = "text length exceeds the limit"))]
    text: Option<String>,
    completed: Option<bool>,
}

#[cfg(test)]
impl Todo {
    pub fn new(id: i32, text: String) -> Self {
        Self {
            id,
            text,
            completed: false,
        }
    }
}
