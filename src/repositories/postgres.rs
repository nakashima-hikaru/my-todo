use sqlx::PgPool;

#[derive(Debug, Clone)]
pub(crate) struct PostgresRepository {
    pub(crate) pool: PgPool,
}

impl PostgresRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
