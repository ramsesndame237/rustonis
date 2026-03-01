use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{CacheError, CacheStore};

// ─── Internal entry ───────────────────────────────────────────────────────────

struct Entry {
    data:       Vec<u8>,
    expires_at: Option<Instant>,
}

impl Entry {
    fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |exp| Instant::now() > exp)
    }
}

// ─── InMemoryStore ────────────────────────────────────────────────────────────

/// Thread-safe, TTL-aware in-memory cache.
///
/// Best suited for **single-instance** deployments, development and tests.
/// Values are discarded when the process exits.
///
/// ```rust
/// use rustonis_cache::{InMemoryStore, CacheStore};
/// use std::time::Duration;
///
/// # tokio_test::block_on(async {
/// let store = InMemoryStore::new();
/// store.put_raw("hello", b"world".to_vec(), Some(Duration::from_secs(60))).await.unwrap();
/// assert_eq!(store.get_raw("hello").await, Some(b"world".to_vec()));
/// # });
/// ```
#[derive(Clone, Default)]
pub struct InMemoryStore {
    map: Arc<RwLock<HashMap<String, Entry>>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl CacheStore for InMemoryStore {
    async fn get_raw(&self, key: &str) -> Option<Vec<u8>> {
        let map = self.map.read().await;
        let entry = map.get(key)?;
        if entry.is_expired() {
            return None;
        }
        Some(entry.data.clone())
    }

    async fn put_raw(
        &self,
        key: &str,
        value: Vec<u8>,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError> {
        let expires_at = ttl.map(|d| Instant::now() + d);
        let mut map = self.map.write().await;
        map.insert(key.to_string(), Entry { data: value, expires_at });
        Ok(())
    }

    async fn forget(&self, key: &str) -> Result<(), CacheError> {
        self.map.write().await.remove(key);
        Ok(())
    }

    async fn has(&self, key: &str) -> bool {
        let map = self.map.read().await;
        map.get(key).map_or(false, |e| !e.is_expired())
    }

    async fn flush(&self) -> Result<(), CacheError> {
        self.map.write().await.clear();
        Ok(())
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_put_and_get_raw() {
        let store = InMemoryStore::new();
        store.put_raw("k", b"v".to_vec(), None).await.unwrap();
        assert_eq!(store.get_raw("k").await, Some(b"v".to_vec()));
    }

    #[tokio::test]
    async fn test_get_missing_key_returns_none() {
        let store = InMemoryStore::new();
        assert_eq!(store.get_raw("missing").await, None);
    }

    #[tokio::test]
    async fn test_expired_entry_returns_none() {
        let store = InMemoryStore::new();
        // Store with 1 nanosecond TTL — already expired by the time we read
        store
            .put_raw("k", b"v".to_vec(), Some(Duration::from_nanos(1)))
            .await
            .unwrap();
        // Spin briefly so the entry expires
        std::thread::sleep(std::time::Duration::from_millis(1));
        assert_eq!(store.get_raw("k").await, None);
    }

    #[tokio::test]
    async fn test_has_returns_true_for_live_entry() {
        let store = InMemoryStore::new();
        store.put_raw("k", b"v".to_vec(), None).await.unwrap();
        assert!(store.has("k").await);
    }

    #[tokio::test]
    async fn test_has_returns_false_for_missing_key() {
        let store = InMemoryStore::new();
        assert!(!store.has("nope").await);
    }

    #[tokio::test]
    async fn test_forget_removes_entry() {
        let store = InMemoryStore::new();
        store.put_raw("k", b"v".to_vec(), None).await.unwrap();
        store.forget("k").await.unwrap();
        assert_eq!(store.get_raw("k").await, None);
    }

    #[tokio::test]
    async fn test_flush_clears_all_entries() {
        let store = InMemoryStore::new();
        store.put_raw("a", b"1".to_vec(), None).await.unwrap();
        store.put_raw("b", b"2".to_vec(), None).await.unwrap();
        store.flush().await.unwrap();
        assert!(!store.has("a").await);
        assert!(!store.has("b").await);
    }

    #[tokio::test]
    async fn test_overwrite_existing_key() {
        let store = InMemoryStore::new();
        store.put_raw("k", b"old".to_vec(), None).await.unwrap();
        store.put_raw("k", b"new".to_vec(), None).await.unwrap();
        assert_eq!(store.get_raw("k").await, Some(b"new".to_vec()));
    }
}
