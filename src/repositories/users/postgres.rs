use axum::async_trait;

use crate::repositories::postgres::PostgresRepository;
use crate::repositories::users::{CreateUser, UpdateUser, User, UserRepository};
use crate::repositories::RepositoryError;

#[async_trait]
impl UserRepository for PostgresRepository {
    async fn create(&self, payload: CreateUser) -> anyhow::Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users(username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            payload.username,
            payload.email,
            payload.password_hash
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> anyhow::Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"
                SELECT id, username, email, password_hash
                FROM users
                WHERE email = $1
                "#,
            email
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                RepositoryError::NotFound("email".to_string(), email.to_string())
            }
            _ => RepositoryError::Unexpected(e.to_string()),
        })?;
        Ok(user)
    }

    async fn find_by_id(&self, id: i32) -> anyhow::Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"
                SELECT id, username, email, password_hash
                FROM users
                WHERE id = $1
                "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound("id".to_string(), id),
            _ => RepositoryError::Unexpected(e.to_string()),
        })?;
        Ok(user)
    }

    async fn update(&self, id: i32, payload: UpdateUser) -> anyhow::Result<User> {
        let old_user = self.find_by_id(id).await?;
        let todo = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET
            username = $1, email = $2, password_hash = $3
            WHERE id = $4
            RETURNING *
            "#,
            payload.username.unwrap_or(old_user.username),
            payload.email.unwrap_or(old_user.email),
            payload.password_hash.unwrap_or(old_user.password_hash),
            id,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(todo)
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM users
            WHERE id = $1
            "#,
            id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound("id".to_string(), id),
            _ => RepositoryError::Unexpected(e.to_string()),
        })?;
        Ok(())
    }
}
