use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{Mutex, Notify};

use crate::job::QueuedJob;
use crate::{Job, JobId};

// ─── InMemoryQueue ────────────────────────────────────────────────────────────

/// In-memory queue backed by a `HashMap<queue_name, VecDeque<job>>`.
///
/// Use [`Dispatcher`](crate::Dispatcher) as the high-level API; use this struct
/// directly only when you need to hand the queue to a [`Worker`](crate::Worker).
pub struct InMemoryQueue {
    queues: Arc<Mutex<HashMap<String, VecDeque<QueuedJob>>>>,
    notify: Arc<Notify>,
}

impl Default for InMemoryQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryQueue {
    /// Create an empty in-memory queue.
    pub fn new() -> Self {
        Self {
            queues: Arc::new(Mutex::new(HashMap::new())),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Push a job for immediate execution.
    pub async fn push<J: Job>(&self, job: J) -> JobId {
        let queued = QueuedJob::new(Box::new(job));
        let id = queued.id.clone();
        let name = queued.job.queue_name().to_string();

        self.queues.lock().await
            .entry(name)
            .or_default()
            .push_back(queued);

        self.notify.notify_waiters();
        id
    }

    /// Push a job that should only run after `delay` has elapsed.
    pub async fn push_delayed<J: Job>(&self, job: J, delay: Duration) -> JobId {
        let queued = QueuedJob::new_delayed(Box::new(job), delay);
        let id = queued.id.clone();
        let name = queued.job.queue_name().to_string();

        self.queues.lock().await
            .entry(name)
            .or_default()
            .push_back(queued);

        id
    }

    /// Pop the next ready job from the first matching queue name (priority order).
    pub(crate) async fn pop(&self, queue_names: &[String]) -> Option<QueuedJob> {
        let mut queues = self.queues.lock().await;
        for name in queue_names {
            let q = queues.entry(name.clone()).or_default();
            if let Some(pos) = q.iter().position(|j| j.is_ready()) {
                return q.remove(pos);
            }
        }
        None
    }

    /// Re-enqueue a failed job after a back-off delay (used by the Worker on retry).
    pub(crate) async fn requeue(&self, mut queued: QueuedJob, delay: Duration) {
        queued.run_at = tokio::time::Instant::now() + delay;
        let name = queued.job.queue_name().to_string();
        self.queues.lock().await
            .entry(name)
            .or_default()
            .push_back(queued);
    }

    /// Return the total number of jobs pending in `queue_name` (ready + delayed).
    pub async fn size(&self, queue_name: &str) -> usize {
        self.queues.lock().await
            .get(queue_name)
            .map_or(0, |q| q.len())
    }

    /// Obtain the notify handle so a Worker can wait for new jobs.
    pub(crate) fn notify(&self) -> Arc<Notify> {
        self.notify.clone()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Job, JobError};
    use std::time::Duration;

    struct DummyJob;

    #[async_trait::async_trait]
    impl Job for DummyJob {
        async fn handle(&self) -> Result<(), JobError> {
            Ok(())
        }
    }

    struct SlowQueue;

    #[async_trait::async_trait]
    impl Job for SlowQueue {
        async fn handle(&self) -> Result<(), JobError> {
            Ok(())
        }
        fn queue_name(&self) -> &'static str {
            "slow"
        }
    }

    #[tokio::test]
    async fn test_push_and_pop_single() {
        let q = InMemoryQueue::new();
        let id = q.push(DummyJob).await;
        assert_eq!(q.size("default").await, 1);

        let job = q.pop(&[String::from("default")]).await;
        assert!(job.is_some());
        assert_eq!(job.unwrap().id, id);
        assert_eq!(q.size("default").await, 0);
    }

    #[tokio::test]
    async fn test_push_multiple_pop_in_order() {
        let q = InMemoryQueue::new();
        q.push(DummyJob).await;
        q.push(DummyJob).await;
        q.push(DummyJob).await;

        assert_eq!(q.size("default").await, 3);
        q.pop(&[String::from("default")]).await;
        assert_eq!(q.size("default").await, 2);
    }

    #[tokio::test]
    async fn test_delayed_job_not_popped_early() {
        let q = InMemoryQueue::new();
        q.push_delayed(DummyJob, Duration::from_secs(100)).await;

        assert_eq!(q.size("default").await, 1);
        let job = q.pop(&[String::from("default")]).await;
        assert!(job.is_none(), "delayed job should not be ready yet");
        // Job still in queue
        assert_eq!(q.size("default").await, 1);
    }

    #[tokio::test]
    async fn test_named_queue() {
        let q = InMemoryQueue::new();
        q.push(SlowQueue).await;

        assert_eq!(q.size("slow").await, 1);
        assert_eq!(q.size("default").await, 0);

        let job = q.pop(&[String::from("slow")]).await;
        assert!(job.is_some());
    }

    #[tokio::test]
    async fn test_pop_respects_priority_order() {
        let q = InMemoryQueue::new();
        q.push(DummyJob).await;  // "default"
        q.push(SlowQueue).await; // "slow"

        // Priority: default first
        let j = q.pop(&[String::from("default"), String::from("slow")]).await;
        assert!(j.is_some());
        assert_eq!(q.size("default").await, 0);
        assert_eq!(q.size("slow").await, 1);
    }

    #[tokio::test]
    async fn test_size_empty_queue() {
        let q = InMemoryQueue::new();
        assert_eq!(q.size("nonexistent").await, 0);
    }
}
