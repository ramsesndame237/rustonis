use std::convert::Infallible;

use axum::{body::Body, response::IntoResponse, routing::Route, Router as AxumRouter};
use http::Request;
use tokio::net::TcpListener;
use tower::{Layer, Service};

use crate::router::Router;

/// Serveur HTTP Rustonis.
///
/// Wrapping d'`axum::serve` avec une API plus simple pour le démarrage
/// et la configuration des middleware globaux (`start/kernel.rs`).
///
/// # Exemple
///
/// ```rust,ignore
/// use rustonis_http::{HttpServer, Router};
/// use tower_http::trace::TraceLayer;
///
/// let router = Router::new().get("/", || async { "Hello" });
///
/// HttpServer::new(router)
///     .layer(TraceLayer::new_for_http())
///     .serve(3333)
///     .await
///     .expect("Server failed");
/// ```
pub struct HttpServer {
    router: AxumRouter,
}

impl HttpServer {
    /// Crée un `HttpServer` à partir d'un `Router` Rustonis.
    pub fn new(router: Router) -> Self {
        Self {
            router: router.into_axum(),
        }
    }

    /// Applique un middleware Tower global sur toutes les routes.
    ///
    /// Utilisation typique dans `start/kernel.rs` :
    /// ```rust,ignore
    /// use tower_http::{cors::CorsLayer, trace::TraceLayer};
    ///
    /// pub fn register(server: HttpServer) -> HttpServer {
    ///     server
    ///         .layer(TraceLayer::new_for_http())
    ///         .layer(CorsLayer::permissive())
    /// }
    /// ```
    pub fn layer<L>(self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<Request<Body>> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request<Body>>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request<Body>>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request<Body>>>::Future: Send + 'static,
    {
        Self {
            router: self.router.layer(layer),
        }
    }

    /// Démarre le serveur sur le port donné.
    ///
    /// Bloque jusqu'à la fin (signal d'arrêt ou erreur).
    pub async fn serve(self, port: u16) -> Result<(), std::io::Error> {
        let addr = format!("0.0.0.0:{port}");
        let listener = TcpListener::bind(&addr).await?;
        tracing::info!("🦀 Rustonis server listening on http://{addr}");
        axum::serve(listener, self.router).await
    }
}
