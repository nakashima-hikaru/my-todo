use axum::async_trait;
use sqlx::PgPool;

use crate::repositories::{CreateTodo, RepositoryError, Todo, TodoRepository, UpdateTodo};

#[derive(Debug, Clone)]
pub struct DatabaseRepository {
    pool: PgPool,
}

impl DatabaseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TodoRepository for DatabaseRepository {
    async fn create(&self, payload: CreateTodo) -> anyhow::Result<Todo> {
        let todo = sqlx::query_as::<_, Todo>(
            r#"
            insert into todos (text, completed) values ($1, false)
            returning *
            "#
        ).bind(payload.text).fetch_one(&self.pool).await?;
        Ok(todo)
    }

    async fn find(&self, id: i32) -> anyhow::Result<Todo> {
        let todo = sqlx::query_as::<_, Todo>(
            r#"
            select * from todos
            where id = $1
            "#
        ).bind(id).fetch_one(&self.pool).await.map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
            _ => RepositoryError::Unexpected(e.to_string())
        })?;
        Ok(todo)
    }

    async fn all(&self) -> anyhow::Result<Vec<Todo>> {
        let todo = sqlx::query_as::<_, Todo>(
            r#"
            select * from todos
            order by id desc
            "#
        ).fetch_all(&self.pool).await?;
        Ok(todo)
    }

    async fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo> {
        let old_todo = self.find(id).await?;
        let todo = sqlx::query_as::<_, Todo>(
            r#"
            update todos
            set
            text = $1, completed = $2
            where id = $3
            returning *
            "#
        ).bind(payload.text.unwrap_or(old_todo.text))
            .bind(payload.completed.unwrap_or(old_todo.completed))
            .bind(id).fetch_one(&self.pool).await?;
        Ok(todo)
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            delete from todos
            where id = $1
            "#
        ).bind(id).fetch_one(&self.pool).await.map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
            _ => RepositoryError::Unexpected(e.to_string())
        })?;
        Ok(())
    }
}