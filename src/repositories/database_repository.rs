use axum::async_trait;
use sqlx::PgPool;

use crate::repositories::{CreateTodo, RepositoryError, Todo, TodoRepository, UpdateTodo};

#[derive(Debug, Clone)]
pub(crate) struct DatabaseRepository {
    pool: PgPool,
}

impl DatabaseRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TodoRepository for DatabaseRepository {
    async fn create(&self, payload: CreateTodo) -> anyhow::Result<Todo> {
        let todo = sqlx::query_as!(
            Todo,
            r#"
            insert into todos (text, completed) values ($1, false)
            returning *
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
            select * from todos
            where id = $1
            "#,
            id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
            _ => RepositoryError::Unexpected(e.to_string()),
        })?;
        Ok(todo)
    }

    async fn all(&self) -> anyhow::Result<Vec<Todo>> {
        let todo = sqlx::query_as!(
            Todo,
            r#"
            select * from todos
            order by id desc
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
            update todos
            set
            text = $1, completed = $2
            where id = $3
            returning *
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
            delete from todos
            where id = $1
            "#,
            id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
            _ => RepositoryError::Unexpected(e.to_string()),
        })?;
        Ok(())
    }
}
