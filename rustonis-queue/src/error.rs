use thiserror::Error;

/// Errors that can occur during job processing or queue operations.
#[derive(Debug, Error)]
pub enum JobError {
    /// The job handler returned an error.
    #[error("Job execution failed: {0}")]
    Failed(String),

    /// The global [`Dispatcher`](crate::Dispatcher) has not been initialised yet.
    #[error("Queue not initialised — call Dispatcher::init() at application startup")]
    NotInitialized,

    /// A low-level queue operation failed.
    #[error("Queue error: {0}")]
    Queue(String),
}
