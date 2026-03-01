//! Per-IP sliding-window rate limiter — Tower [`Layer`] / [`Service`] pair.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rustonis_http::{Router, RateLimitLayer};
//! use std::time::Duration;
//!
//! let router = Router::new()
//!     .get("/api/hello", || async { "hello" });
//!
//! // Allow at most 100 requests per 60 seconds per IP
//! let _layer = RateLimitLayer::new(100, Duration::from_secs(60));
//! ```

use std::{
    collections::HashMap,
    future::Future,
    net::IpAddr,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::{Duration, Instant},
};

use axum::{body::Body, response::Response};
use http::{Request, StatusCode};
use serde_json::json;
use tower::{Layer, Service};

// ─── Layer ────────────────────────────────────────────────────────────────────

/// Tower [`Layer`] that applies per-IP sliding-window rate limiting.
///
/// Requests exceeding `max_requests` within `window` return **429 Too Many Requests**.
#[derive(Clone)]
pub struct RateLimitLayer {
    max_requests: usize,
    window:       Duration,
    state:        Arc<Mutex<HashMap<IpAddr, Vec<Instant>>>>,
}

impl RateLimitLayer {
    /// Create a new rate limit layer.
    ///
    /// * `max_requests` — maximum requests allowed per IP within `window`.
    /// * `window`       — rolling time window.
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            state: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            max_requests: self.max_requests,
            window:       self.window,
            state:        Arc::clone(&self.state),
        }
    }
}

// ─── Service ──────────────────────────────────────────────────────────────────

/// Tower [`Service`] produced by [`RateLimitLayer`].
#[derive(Clone)]
pub struct RateLimitService<S> {
    inner:        S,
    max_requests: usize,
    window:       Duration,
    state:        Arc<Mutex<HashMap<IpAddr, Vec<Instant>>>>,
}

impl<S> RateLimitService<S> {
    /// Check whether `ip` is within the rate limit.
    ///
    /// Returns `true` if the request is allowed, `false` if it should be
    /// rejected. Internally prunes timestamps older than `window` and records
    /// the current request time.
    fn is_allowed(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let mut map = self.state.lock().expect("rate-limit mutex poisoned");
        let timestamps = map.entry(ip).or_default();

        // Prune expired entries
        timestamps.retain(|&t| now.duration_since(t) < self.window);

        if timestamps.len() < self.max_requests {
            timestamps.push(now);
            true
        } else {
            false
        }
    }
}

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

impl<S> Service<Request<Body>> for RateLimitService<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        // Extract the client IP from the `X-Forwarded-For` header or
        // `X-Real-IP` header; fall back to `0.0.0.0` when running in tests
        // where no socket address is available.
        let ip = extract_ip(&req);

        if !self.is_allowed(ip) {
            let body = json!({
                "error":   "Too Many Requests",
                "message": "Rate limit exceeded. Please slow down."
            })
            .to_string();

            let response = Response::builder()
                .status(StatusCode::TOO_MANY_REQUESTS)
                .header("Content-Type", "application/json")
                .body(Body::from(body))
                .expect("infallible response build");

            return Box::pin(async move { Ok(response) });
        }

        let fut = self.inner.call(req);
        Box::pin(async move { fut.await })
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn extract_ip(req: &Request<Body>) -> IpAddr {
    // X-Forwarded-For (first hop) or X-Real-IP
    let header_ip = req
        .headers()
        .get("x-forwarded-for")
        .or_else(|| req.headers().get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<IpAddr>().ok());

    header_ip.unwrap_or_else(|| "0.0.0.0".parse().unwrap())
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, routing::get};
    use http::{Request, StatusCode};
    use tower::{Service, ServiceExt};

    async fn ok_handler() -> &'static str {
        "ok"
    }

    fn make_request(ip: &str) -> Request<Body> {
        Request::builder()
            .uri("/")
            .header("x-forwarded-for", ip)
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn test_requests_within_limit_are_allowed() {
        let layer = RateLimitLayer::new(3, Duration::from_secs(60));
        let axum_router = axum::Router::new().route("/", get(ok_handler));
        let mut svc = layer.layer(axum_router);

        for _ in 0..3 {
            let res = svc.ready().await.unwrap().call(make_request("1.2.3.4")).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn test_request_over_limit_returns_429() {
        let layer = RateLimitLayer::new(2, Duration::from_secs(60));
        let axum_router = axum::Router::new().route("/", get(ok_handler));
        let mut svc = layer.layer(axum_router);

        // First two are OK
        for _ in 0..2 {
            let res = svc.ready().await.unwrap().call(make_request("10.0.0.1")).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
        }

        // Third exceeds limit
        let res = svc.ready().await.unwrap().call(make_request("10.0.0.1")).await.unwrap();
        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn test_different_ips_have_independent_limits() {
        let layer = RateLimitLayer::new(1, Duration::from_secs(60));
        let axum_router = axum::Router::new().route("/", get(ok_handler));
        let mut svc = layer.layer(axum_router);

        // IP A uses its quota
        let res = svc.ready().await.unwrap().call(make_request("192.168.1.1")).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let res = svc.ready().await.unwrap().call(make_request("192.168.1.1")).await.unwrap();
        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);

        // IP B still has its own quota
        let res = svc.ready().await.unwrap().call(make_request("192.168.1.2")).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_429_response_has_json_content_type() {
        let layer = RateLimitLayer::new(0, Duration::from_secs(60)); // 0 = always reject
        let axum_router = axum::Router::new().route("/", get(ok_handler));
        let mut svc = layer.layer(axum_router);

        let res = svc.ready().await.unwrap().call(make_request("5.5.5.5")).await.unwrap();
        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "application/json"
        );
    }

    #[tokio::test]
    async fn test_window_expiry_resets_counter() {
        // Use a 1 ns window so all entries expire immediately
        let layer = RateLimitLayer::new(1, Duration::from_nanos(1));
        let axum_router = axum::Router::new().route("/", get(ok_handler));
        let mut svc = layer.layer(axum_router);

        // First request is within quota
        let res = svc.ready().await.unwrap().call(make_request("7.7.7.7")).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // Sleep briefly so the window expires
        std::thread::sleep(Duration::from_millis(1));

        // After window expiry, quota is reset — request is allowed again
        let res = svc.ready().await.unwrap().call(make_request("7.7.7.7")).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
