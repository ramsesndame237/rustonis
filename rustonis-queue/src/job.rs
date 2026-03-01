use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::JobError;

// ─── JobId ────────────────────────────────────────────────────────────────────

static JOB_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Unique identifier assigned to every dispatched job.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JobId(pub u64);

impl JobId {
    pub(crate) fn new() -> Self {
        Self(JOB_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "job-{}", self.0)
    }
}

// ─── Job trait ────────────────────────────────────────────────────────────────

/// A unit of asynchronous work processed by a [`Worker`](crate::Worker).
///
/// Implement this trait for each job type and dispatch via [`Dispatcher`](crate::Dispatcher).
///
/// ```rust,no_run
/// use rustonis_queue::{Job, JobError};
///
/// pub struct SendWelcomeEmail {
///     pub user_id: u64,
/// }
///
/// #[async_trait::async_trait]
/// impl Job for SendWelcomeEmail {
///     async fn handle(&self) -> Result<(), JobError> {
///         println!("Sending welcome email to user {}", self.user_id);
///         Ok(())
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait Job: Send + Sync + 'static {
    /// Execute the job.  Return `Err` to trigger a retry (up to [`max_attempts`](Job::max_attempts)).
    async fn handle(&self) -> Result<(), JobError>;

    /// Maximum number of execution attempts before the job is marked as permanently failed.
    /// Defaults to `3`.
    fn max_attempts(&self) -> u32 {
        3
    }

    /// Name of the queue this job belongs to.  Defaults to `"default"`.
    fn queue_name(&self) -> &'static str {
        "default"
    }
}

// ─── QueuedJob (internal) ─────────────────────────────────────────────────────

/// Internal wrapper stored in the queue.
pub(crate) struct QueuedJob {
    pub id:       JobId,
    pub job:      Box<dyn Job>,
    pub attempts: u32,
    /// Monotonic instant at which this job becomes eligible for processing.
    pub run_at:   tokio::time::Instant,
}

impl QueuedJob {
    pub(crate) fn new(job: Box<dyn Job>) -> Self {
        Self {
            id:       JobId::new(),
            job,
            attempts: 0,
            run_at:   tokio::time::Instant::now(),
        }
    }

    pub(crate) fn new_delayed(job: Box<dyn Job>, delay: Duration) -> Self {
        Self {
            id:       JobId::new(),
            job,
            attempts: 0,
            run_at:   tokio::time::Instant::now() + delay,
        }
    }

    /// Returns `true` when the job's scheduled run-at time has passed.
    pub(crate) fn is_ready(&self) -> bool {
        tokio::time::Instant::now() >= self.run_at
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_id_is_unique() {
        let a = JobId::new();
        let b = JobId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn test_job_id_display() {
        let id = JobId(42);
        assert_eq!(id.to_string(), "job-42");
    }

    #[tokio::test]
    async fn test_queued_job_is_ready_immediately() {
        struct Noop;
        #[async_trait::async_trait]
        impl Job for Noop {
            async fn handle(&self) -> Result<(), JobError> {
                Ok(())
            }
        }

        let queued = QueuedJob::new(Box::new(Noop));
        assert!(queued.is_ready());
    }

    #[tokio::test]
    async fn test_queued_job_delayed_not_ready() {
        struct Noop;
        #[async_trait::async_trait]
        impl Job for Noop {
            async fn handle(&self) -> Result<(), JobError> {
                Ok(())
            }
        }

        let delay = std::time::Duration::from_secs(100);
        let queued = QueuedJob::new_delayed(Box::new(Noop), delay);
        assert!(!queued.is_ready());
    }
}
