use axum::async_trait;

use crate::repositories::postgres::PostgresRepository;
use crate::repositories::todos::{CreateTodo, Todo, TodoRepository, UpdateTodo};
use crate::repositories::RepositoryError;

#[async_trait]
impl TodoRepository for PostgresRepository {
    async fn create(&self, payload: CreateTodo) -> anyhow::Result<Todo> {
        let todo = sqlx::query_as!(
            Todo,
            r#"
            INSERT INTO todos (text, completed) VALUES ($1, false)
            RETURNING *
            "#,
            payload.text,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(todo)
    }

    async fn find(&self, id: i32) -> anyhow::Result<Todo> {
        let todo = sqlx::query_as!(
            Todo,
            r#"
            SELECT * FROM todos
            WHERE id = $1
            "#,
            id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound("id".to_string(), id),
            _ => RepositoryError::Unexpected(e.to_string()),
        })?;
        Ok(todo)
    }

    async fn all(&self) -> anyhow::Result<Vec<Todo>> {
        let todo = sqlx::query_as!(
            Todo,
            r#"
            SELECT * FROM todos
            ORDER BY id DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(todo)
    }

    async fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo> {
        let old_todo = self.find(id).await?;
        let todo = sqlx::query_as!(
            Todo,
            r#"
            UPDATE todos
            SET
            text = $1, completed = $2
            WHERE id = $3
            RETURNING *
            "#,
            payload.text.unwrap_or(old_todo.text),
            payload.completed.unwrap_or(old_todo.completed),
            id,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(todo)
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM todos
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
