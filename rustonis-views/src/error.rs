use thiserror::Error;

#[derive(Debug, Error)]
pub enum ViewError {
    #[error("Template engine error: {0}")]
    Tera(#[from] tera::Error),

    #[error("Template not found: {0}")]
    NotFound(String),

    #[error("View engine not initialized — call View::init() first")]
    NotInitialized,
}
