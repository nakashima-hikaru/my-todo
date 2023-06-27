use thiserror::Error;

pub(crate) mod hash_map;
pub(crate) mod postgres;
pub(crate) mod todos;
mod users;

#[derive(Debug, Error)]
enum RepositoryError<T> {
    #[error("Unexpected error: [{0}]")]
    Unexpected(String),
    #[error("NotFound, {0}: {1}")]
    NotFound(String, T),
}
