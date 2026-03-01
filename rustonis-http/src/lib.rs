pub mod error;
pub mod ratelimit;
pub mod response;
pub mod router;
pub mod server;

pub use error::AppError;
pub use ratelimit::RateLimitLayer;
pub use response::{JsonResponse, NoContent};
pub use router::Router;
pub use server::HttpServer;

#[cfg(test)]
mod tests {
    use super::*;

    use axum::body::Body;
    use http::{Request, StatusCode};
    use tower::ServiceExt; // oneshot

    // ─── Handlers de test ───────────────────────────────────────────────────

    async fn hello_handler() -> JsonResponse<&'static str> {
        JsonResponse::ok("world")
    }

    async fn created_handler() -> JsonResponse<serde_json::Value> {
        JsonResponse::created(serde_json::json!({ "id": 1, "name": "Alice" }))
    }

    async fn not_found_handler() -> Result<JsonResponse<()>, AppError> {
        Err(AppError::not_found("Resource not found"))
    }

    async fn unauth_handler() -> Result<JsonResponse<()>, AppError> {
        Err(AppError::unauthorized("Token missing"))
    }

    async fn validation_handler() -> Result<JsonResponse<()>, AppError> {
        let mut errors = std::collections::HashMap::new();
        errors.insert("email".to_string(), vec!["is invalid".to_string()]);
        Err(AppError::validation("Validation failed", errors))
    }

    async fn no_content_handler() -> NoContent {
        NoContent
    }

    // ─── Helpers ────────────────────────────────────────────────────────────

    async fn call(router: Router, uri: &str, method: &str) -> http::Response<Body> {
        let app = router.into_axum();
        let request = Request::builder()
            .uri(uri)
            .method(method)
            .body(Body::empty())
            .unwrap();
        app.oneshot(request).await.unwrap()
    }

    async fn body_json(response: http::Response<Body>) -> serde_json::Value {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    // ─── Tests Router ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_route_returns_200() {
        let router = Router::new().get("/hello", hello_handler);
        let res = call(router, "/hello", "GET").await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_route_returns_correct_body() {
        let router = Router::new().get("/hello", hello_handler);
        let res = call(router, "/hello", "GET").await;
        let body = body_json(res).await;
        assert_eq!(body, "world");
    }

    #[tokio::test]
    async fn test_post_route_returns_201() {
        let router = Router::new().post("/users", created_handler);
        let app = router.into_axum();
        let request = Request::builder()
            .uri("/users")
            .method("POST")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(request).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_wrong_method_returns_405() {
        let router = Router::new().get("/hello", hello_handler);
        let res = call(router, "/hello", "POST").await;
        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_unknown_route_returns_404() {
        let router = Router::new().get("/hello", hello_handler);
        let res = call(router, "/not-here", "GET").await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_route_group_with_prefix() {
        let router = Router::new().group("/api/v1", |r| {
            r.get("/users", hello_handler)
        });

        // Route avec préfixe — doit fonctionner
        let res = call(router, "/api/v1/users", "GET").await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_route_group_prefix_not_accessible_without_prefix() {
        let router = Router::new().group("/api/v1", |r| {
            r.get("/users", hello_handler)
        });

        // Route sans préfixe — ne doit PAS fonctionner
        let res = call(router, "/users", "GET").await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_router_merge() {
        let r1 = Router::new().get("/foo", hello_handler);
        let r2 = Router::new().get("/bar", hello_handler);
        let router = Router::new().merge(r1).merge(r2);

        let res_foo = call(Router::new().merge(router), "/foo", "GET").await;
        // Note: router consumed above, use separate routers for each call
        assert_eq!(res_foo.status(), StatusCode::OK);
    }

    // ─── Tests JsonResponse ───────────────────────────────────────────────────

    #[tokio::test]
    async fn test_json_response_ok_status_and_body() {
        let router = Router::new().get("/", hello_handler);
        let res = call(router, "/", "GET").await;
        assert_eq!(res.status(), StatusCode::OK);
        let body = body_json(res).await;
        assert_eq!(body.as_str().unwrap(), "world");
    }

    #[tokio::test]
    async fn test_json_response_created_status() {
        let router = Router::new().post("/", created_handler);
        let app = router.into_axum();
        let req = Request::builder()
            .uri("/")
            .method("POST")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
        let body = body_json(res).await;
        assert_eq!(body["name"], "Alice");
    }

    #[tokio::test]
    async fn test_no_content_returns_204() {
        let router = Router::new().delete("/", no_content_handler);
        let app = router.into_axum();
        let req = Request::builder()
            .uri("/")
            .method("DELETE")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }

    // ─── Tests AppError ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_app_error_not_found_returns_404() {
        let router = Router::new().get("/", not_found_handler);
        let res = call(router, "/", "GET").await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        let body = body_json(res).await;
        assert_eq!(body["error"], "Not Found");
        assert_eq!(body["message"], "Resource not found");
    }

    #[tokio::test]
    async fn test_app_error_unauthorized_returns_401() {
        let router = Router::new().get("/", unauth_handler);
        let res = call(router, "/", "GET").await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        let body = body_json(res).await;
        assert_eq!(body["error"], "Unauthorized");
    }

    #[tokio::test]
    async fn test_app_error_validation_returns_422() {
        let router = Router::new().get("/", validation_handler);
        let res = call(router, "/", "GET").await;
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = body_json(res).await;
        assert_eq!(body["error"], "Unprocessable Entity");
        assert!(body["errors"]["email"].is_array());
    }

    // ─── Test CRUD complet sur un groupe préfixé ──────────────────────────────

    #[tokio::test]
    async fn test_full_crud_route_group() {
        async fn list() -> JsonResponse<Vec<u32>> {
            JsonResponse::ok(vec![1, 2, 3])
        }
        async fn create() -> JsonResponse<u32> {
            JsonResponse::created(42)
        }
        async fn destroy() -> NoContent {
            NoContent
        }

        // Routes groupées sous /api/v1 (non-root routes dans le groupe)
        let router = Router::new().group("/api/v1", |r| {
            r.get("/items", list)
             .post("/items", create)
             .delete("/items/1", destroy)
        });

        let app = router.into_axum();

        // GET /api/v1/items
        let res = app
            .clone()
            .oneshot(Request::builder().uri("/api/v1/items").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // POST /api/v1/items
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/items")
                    .method("POST")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);

        // DELETE /api/v1/items/1
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/items/1")
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }
}
