use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrmError {
    #[error("Database connection error: {0}")]
    Connection(#[source] sqlx::Error),

    #[error("Query error: {0}")]
    Query(#[source] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Record not found")]
    NotFound,

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<sqlx::Error> for OrmError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => OrmError::NotFound,
            other => OrmError::Query(other),
        }
    }
}
