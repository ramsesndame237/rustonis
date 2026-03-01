//! `rustonis-queue` — Async job queue for the Rustonis framework.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use rustonis_queue::{Dispatcher, InMemoryQueue, Job, JobError, Worker};
//!
//! // 1. Define a job
//! pub struct SendEmail { pub to: String }
//!
//! #[async_trait::async_trait]
//! impl Job for SendEmail {
//!     async fn handle(&self) -> Result<(), JobError> {
//!         println!("Sending email to {}", self.to);
//!         Ok(())
//!     }
//! }
//!
//! # async fn run() -> Result<(), JobError> {
//! // 2. Boot the dispatcher (once, at startup)
//! Dispatcher::init(InMemoryQueue::new());
//!
//! // 3. Spawn a worker
//! let queue = Dispatcher::queue_backend()?;
//! tokio::spawn(Worker::new(queue).concurrency(4).run());
//!
//! // 4. Dispatch jobs from anywhere
//! Dispatcher::dispatch(SendEmail { to: "user@example.com".into() }).await?;
//! # Ok(())
//! # }
//! ```

pub mod dispatcher;
pub mod error;
pub mod job;
pub mod queue;
pub mod worker;

pub use dispatcher::Dispatcher;
pub use error::JobError;
pub use job::{Job, JobId};
pub use queue::InMemoryQueue;
pub use worker::Worker;

/// Convenience re-export for `use rustonis_queue::prelude::*`.
pub mod prelude {
    pub use crate::{Dispatcher, InMemoryQueue, Job, JobError, JobId, Worker};
}
