use std::time::Duration;

use async_trait::async_trait;

use crate::CacheError;

/// Low-level cache driver interface.
///
/// Implement this trait to add a new cache backend (Redis, Memcached, DB …).
/// All values are opaque byte vectors; serialization is handled by the
/// higher-level [`crate::Cache`] façade.
#[async_trait]
pub trait CacheStore: Send + Sync {
    /// Return the raw bytes stored under `key`, or `None` if missing / expired.
    async fn get_raw(&self, key: &str) -> Option<Vec<u8>>;

    /// Store `value` under `key` with an optional TTL.
    /// `ttl = None` means the entry never expires.
    async fn put_raw(
        &self,
        key: &str,
        value: Vec<u8>,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError>;

    /// Delete the entry for `key` (no-op if absent).
    async fn forget(&self, key: &str) -> Result<(), CacheError>;

    /// Return `true` if `key` exists and has not expired.
    async fn has(&self, key: &str) -> bool;

    /// Remove **all** entries from the store.
    async fn flush(&self) -> Result<(), CacheError>;
}
