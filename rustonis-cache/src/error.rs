use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Cache store error: {0}")]
    Store(String),

    /// Returned when [`crate::Cache::init`] has not been called yet.
    #[error("Cache store is not initialized — call Cache::init() first")]
    NotInitialized,
}
