//! `rustonis-cache` — Caching for Rustonis.
//!
//! ## Quick Start
//!
//! ```rust
//! use rustonis_cache::{Cache, InMemoryStore};
//! use std::{sync::Arc, time::Duration};
//!
//! # tokio_test::block_on(async {
//! // Initialize once at application startup (e.g. in a ServiceProvider)
//! Cache::init(Arc::new(InMemoryStore::new()));
//!
//! // Store a value for 5 minutes
//! Cache::put("greeting", &"hello world", Some(Duration::from_secs(300)))
//!     .await
//!     .unwrap();
//!
//! // Retrieve it
//! let v: Option<String> = Cache::get("greeting").await;
//! assert_eq!(v.as_deref(), Some("hello world"));
//! # });
//! ```
//!
//! ## remember()
//!
//! Fetch from cache or compute (and cache) the value in one call:
//!
//! ```rust
//! use rustonis_cache::{Cache, CacheError, InMemoryStore};
//! use std::{sync::Arc, time::Duration};
//!
//! # tokio_test::block_on(async {
//! Cache::init(Arc::new(InMemoryStore::new()));
//!
//! let count: u32 = Cache::remember("count", Duration::from_secs(60), || async {
//!     Ok::<u32, CacheError>(42)
//! })
//! .await
//! .unwrap();
//!
//! assert_eq!(count, 42);
//! # });
//! ```

mod error;
mod memory;
mod store;

pub use error::CacheError;
pub use memory::InMemoryStore;
pub use store::CacheStore;

use std::{future::Future, sync::Arc, time::Duration};

use serde::{de::DeserializeOwned, Serialize};

pub mod prelude {
    pub use super::{Cache, CacheError, CacheStore, InMemoryStore};
}

// ─── Global façade ────────────────────────────────────────────────────────────

static STORE: std::sync::OnceLock<Arc<dyn CacheStore>> = std::sync::OnceLock::new();

/// Static façade for the application-wide cache store.
///
/// Call [`Cache::init`] once during application boot before using any other
/// method. All methods silently return `None` / `Err(CacheError::NotInitialized)`
/// if the store has not been initialized.
pub struct Cache;

impl Cache {
    /// Register the global cache store. Must be called once before any
    /// get/put operations, typically in a `ServiceProvider::boot()`.
    pub fn init(store: Arc<dyn CacheStore>) {
        STORE.set(store).ok();
    }

    fn store() -> Result<&'static Arc<dyn CacheStore>, CacheError> {
        STORE.get().ok_or(CacheError::NotInitialized)
    }

    // ─── Read ─────────────────────────────────────────────────────────────────

    /// Deserialize and return the value for `key`, or `None` if missing/expired.
    pub async fn get<T: DeserializeOwned>(key: &str) -> Option<T> {
        let bytes = Self::store().ok()?.get_raw(key).await?;
        serde_json::from_slice(&bytes).ok()
    }

    // ─── Write ────────────────────────────────────────────────────────────────

    /// Serialize `value` and store it under `key`.
    /// Pass `ttl = None` for an entry that never expires.
    pub async fn put<T: Serialize>(
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError> {
        let bytes = serde_json::to_vec(value)?;
        Self::store()?.put_raw(key, bytes, ttl).await
    }

    /// Store `value` under `key` with **no expiry**.
    pub async fn forever<T: Serialize>(key: &str, value: &T) -> Result<(), CacheError> {
        Self::put(key, value, None).await
    }

    // ─── Delete ───────────────────────────────────────────────────────────────

    /// Remove `key` from the cache (no-op if absent).
    pub async fn forget(key: &str) -> Result<(), CacheError> {
        Self::store()?.forget(key).await
    }

    /// Remove **all** entries from the cache.
    pub async fn flush() -> Result<(), CacheError> {
        Self::store()?.flush().await
    }

    // ─── Existence ────────────────────────────────────────────────────────────

    /// Return `true` if `key` exists and has not expired.
    pub async fn has(key: &str) -> bool {
        match Self::store() {
            Ok(s) => s.has(key).await,
            Err(_) => false,
        }
    }

    // ─── remember ─────────────────────────────────────────────────────────────

    /// Return the cached value for `key`, or compute it with `f`, cache it
    /// for `ttl`, and return the result.
    ///
    /// This is the idiomatic way to cache expensive computations.
    pub async fn remember<T, F, Fut>(
        key: &str,
        ttl: Duration,
        compute: F,
    ) -> Result<T, CacheError>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, CacheError>>,
    {
        if let Some(cached) = Self::get::<T>(key).await {
            return Ok(cached);
        }
        let value = compute().await?;
        Self::put(key, &value, Some(ttl)).await?;
        Ok(value)
    }

    /// Like [`Cache::remember`] but stores the value without expiry.
    pub async fn remember_forever<T, F, Fut>(key: &str, compute: F) -> Result<T, CacheError>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, CacheError>>,
    {
        if let Some(cached) = Self::get::<T>(key).await {
            return Ok(cached);
        }
        let value = compute().await?;
        Self::forever(key, &value).await?;
        Ok(value)
    }
}
