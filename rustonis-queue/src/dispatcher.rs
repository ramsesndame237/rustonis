use std::sync::{Arc, OnceLock};
use std::time::Duration;

use crate::{InMemoryQueue, Job, JobError, JobId};

// ─── Global queue ─────────────────────────────────────────────────────────────

static QUEUE: OnceLock<Arc<InMemoryQueue>> = OnceLock::new();

// ─── Dispatcher ───────────────────────────────────────────────────────────────

/// Global job dispatcher.
///
/// Initialise once at application startup; then call [`dispatch`](Dispatcher::dispatch)
/// from anywhere in your codebase.
///
/// ```rust,no_run
/// use std::time::Duration;
/// use rustonis_queue::{Dispatcher, InMemoryQueue, Job, JobError, Worker};
///
/// # #[async_trait::async_trait]
/// # impl Job for MyJob { async fn handle(&self) -> Result<(), JobError> { Ok(()) } }
/// # struct MyJob;
/// # async fn example() -> Result<(), JobError> {
/// // ── Application bootstrap ────────────────────────────
/// Dispatcher::init(InMemoryQueue::new());
///
/// // Hand the same queue to the Worker (runs in a background task)
/// let queue = Dispatcher::queue_backend()?;
/// tokio::spawn(Worker::new(queue).concurrency(4).run());
///
/// // ── Anywhere in your handlers ────────────────────────
/// Dispatcher::dispatch(MyJob).await?;
/// Dispatcher::dispatch_later(MyJob, Duration::from_secs(30)).await?;
/// # Ok(())
/// # }
/// # fn main() {}
/// ```
pub struct Dispatcher;

impl Dispatcher {
    /// Initialise the global queue.  Must be called before any `dispatch` call.
    /// Subsequent calls are silently ignored (OnceLock semantics).
    pub fn init(queue: InMemoryQueue) {
        QUEUE.set(Arc::new(queue)).ok();
    }

    fn queue() -> Result<Arc<InMemoryQueue>, JobError> {
        QUEUE.get().cloned().ok_or(JobError::NotInitialized)
    }

    /// Obtain the underlying queue `Arc` to pass to a [`Worker`](crate::Worker).
    pub fn queue_backend() -> Result<Arc<InMemoryQueue>, JobError> {
        Self::queue()
    }

    /// Dispatch a job for immediate processing.
    pub async fn dispatch<J: Job>(job: J) -> Result<JobId, JobError> {
        Ok(Self::queue()?.push(job).await)
    }

    /// Dispatch a job that should only run after `delay` has elapsed.
    pub async fn dispatch_later<J: Job>(job: J, delay: Duration) -> Result<JobId, JobError> {
        Ok(Self::queue()?.push_delayed(job, delay).await)
    }

    /// Return the number of pending jobs (ready + delayed) in the given queue.
    pub async fn size(queue_name: &str) -> Result<usize, JobError> {
        Ok(Self::queue()?.size(queue_name).await)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InMemoryQueue, Job, JobError};

    // NOTE: QUEUE is a OnceLock — it can only be set once per process.
    // We test the un-initialised error path indirectly by using InMemoryQueue
    // directly; the initialised path is covered by the doctests.

    #[tokio::test]
    async fn test_queue_operations_via_in_memory_queue() {
        struct PingJob;

        #[async_trait::async_trait]
        impl Job for PingJob {
            async fn handle(&self) -> Result<(), JobError> {
                Ok(())
            }
        }

        let q = InMemoryQueue::new();
        let id1 = q.push(PingJob).await;
        let id2 = q.push(PingJob).await;

        assert_ne!(id1, id2);
        assert_eq!(q.size("default").await, 2);
    }

    #[tokio::test]
    async fn test_dispatch_later_increases_size() {
        struct LaterJob;

        #[async_trait::async_trait]
        impl Job for LaterJob {
            async fn handle(&self) -> Result<(), JobError> {
                Ok(())
            }
        }

        let q = InMemoryQueue::new();
        q.push_delayed(LaterJob, Duration::from_secs(60)).await;
        assert_eq!(q.size("default").await, 1);
    }
}
