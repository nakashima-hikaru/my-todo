use axum::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

mod postgres;

#[derive(Clone, Debug, Deserialize, FromRow, PartialEq, Serialize)]
pub(crate) struct User {
    pub(crate) id: i32,
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) password_hash: String,
}

#[derive(Clone, Debug, Deserialize, FromRow, PartialEq, Serialize)]
pub(crate) struct CreateUser {
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) password_hash: String,
}

#[derive(Clone, Debug, Deserialize, FromRow, PartialEq, Serialize)]
pub(crate) struct UpdateUser {
    pub(crate) username: Option<String>,
    pub(crate) email: Option<String>,
    pub(crate) password_hash: Option<String>,
}

#[async_trait]
pub(crate) trait UserRepository {
    async fn create(&self, payload: CreateUser) -> anyhow::Result<User>;
    async fn find_by_email(&self, email: &str) -> anyhow::Result<User>;
    async fn find_by_id(&self, id: i32) -> anyhow::Result<User>;
    async fn update(&self, id: i32, payload: UpdateUser) -> anyhow::Result<User>;
    async fn delete(&self, id: i32) -> anyhow::Result<()>;
}
