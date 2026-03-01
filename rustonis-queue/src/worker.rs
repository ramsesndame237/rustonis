use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Semaphore;

use crate::job::QueuedJob;
use crate::InMemoryQueue;

// ─── Worker ───────────────────────────────────────────────────────────────────

/// Processes jobs from one or more named queues with bounded concurrency.
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use rustonis_queue::{InMemoryQueue, Worker, Dispatcher};
///
/// # async fn run() {
/// Dispatcher::init(InMemoryQueue::new());
/// let queue = Dispatcher::queue_backend().unwrap();
///
/// Worker::new(queue)
///     .concurrency(8)
///     .queues(vec!["default", "high"])
///     .run()
///     .await;
/// # }
/// ```
pub struct Worker {
    queue:       Arc<InMemoryQueue>,
    queue_names: Vec<String>,
    concurrency: usize,
}

impl Worker {
    /// Create a Worker listening on the `"default"` queue with 4 concurrent slots.
    pub fn new(queue: Arc<InMemoryQueue>) -> Self {
        Self {
            queue,
            queue_names: vec!["default".to_string()],
            concurrency: 4,
        }
    }

    /// Set the maximum number of concurrent job handlers.
    pub fn concurrency(mut self, n: usize) -> Self {
        self.concurrency = n;
        self
    }

    /// Set the queue names to poll, in priority order (highest priority first).
    pub fn queues(mut self, names: Vec<&str>) -> Self {
        self.queue_names = names.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Start the worker event-loop.  Runs indefinitely; typically called in a
    /// `tokio::spawn` or as the last expression in `main`.
    pub async fn run(self) {
        let worker    = Arc::new(self);
        let semaphore = Arc::new(Semaphore::new(worker.concurrency));
        let notify    = worker.queue.notify();

        loop {
            match worker.queue.pop(&worker.queue_names).await {
                Some(queued) => {
                    // Block until a concurrency slot is free
                    let permit = semaphore.clone()
                        .acquire_owned()
                        .await
                        .expect("semaphore unexpectedly closed");

                    let w = worker.clone();
                    tokio::spawn(async move {
                        let _permit = permit; // released on drop
                        w.process(queued).await;
                    });
                }
                None => {
                    // No ready jobs — wait for a push notification or a short poll
                    tokio::select! {
                        _ = notify.notified()                             => {}
                        _ = tokio::time::sleep(Duration::from_millis(100)) => {}
                    }
                }
            }
        }
    }

    async fn process(self: Arc<Self>, mut queued: QueuedJob) {
        queued.attempts += 1;

        match queued.job.handle().await {
            Ok(()) => {
                println!(
                    "[rustonis-queue] {} completed (attempt {})",
                    queued.id, queued.attempts
                );
            }
            Err(e) => {
                if queued.attempts < queued.job.max_attempts() {
                    // Exponential back-off: 2^attempts seconds (capped at 512 s)
                    let secs  = 2u64.pow(queued.attempts.min(9));
                    let delay = Duration::from_secs(secs);
                    eprintln!(
                        "[rustonis-queue] {} failed (attempt {}/{}): {} — retrying in {:?}",
                        queued.id, queued.attempts, queued.job.max_attempts(), e, delay
                    );
                    self.queue.requeue(queued, delay).await;
                } else {
                    eprintln!(
                        "[rustonis-queue] {} permanently failed after {} attempts: {}",
                        queued.id, queued.attempts, e
                    );
                }
            }
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InMemoryQueue;

    #[test]
    fn test_worker_construction() {
        let q = Arc::new(InMemoryQueue::new());
        let worker = Worker::new(q)
            .concurrency(8)
            .queues(vec!["high", "default"]);

        assert_eq!(worker.concurrency, 8);
        assert_eq!(worker.queue_names, vec!["high", "default"]);
    }
}
