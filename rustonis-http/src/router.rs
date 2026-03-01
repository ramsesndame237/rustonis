use axum::{handler::Handler, routing, Router as AxumRouter};

/// Router Rustonis — façade AdonisJS-like sur Axum.
///
/// Supporte les méthodes HTTP standard et les groupes de routes préfixés.
/// En interne, délègue à `axum::Router` (isolation via ADR-001).
///
/// # Exemple
///
/// ```rust,ignore
/// use rustonis_http::Router;
///
/// pub fn register() -> Router {
///     Router::new()
///         .get("/", HomeController::index)
///         .group("/api/v1", |r| {
///             r.get("/users", UsersController::index)
///              .post("/users", UsersController::create)
///              .get("/users/:id", UsersController::show)
///              .put("/users/:id", UsersController::update)
///              .delete("/users/:id", UsersController::destroy)
///         })
/// }
/// ```
pub struct Router {
    inner: AxumRouter,
}

impl Router {
    /// Crée un router vide.
    pub fn new() -> Self {
        Self {
            inner: AxumRouter::new(),
        }
    }

    /// Route `GET path → handler`.
    pub fn get<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        Self {
            inner: self.inner.route(path, routing::get(handler)),
        }
    }

    /// Route `POST path → handler`.
    pub fn post<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        Self {
            inner: self.inner.route(path, routing::post(handler)),
        }
    }

    /// Route `PUT path → handler`.
    pub fn put<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        Self {
            inner: self.inner.route(path, routing::put(handler)),
        }
    }

    /// Route `PATCH path → handler`.
    pub fn patch<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        Self {
            inner: self.inner.route(path, routing::patch(handler)),
        }
    }

    /// Route `DELETE path → handler`.
    pub fn delete<H, T>(self, path: &str, handler: H) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        Self {
            inner: self.inner.route(path, routing::delete(handler)),
        }
    }

    /// Groupe de routes sous un préfixe commun.
    ///
    /// ```rust,ignore
    /// Router::new()
    ///     .group("/api/v1", |r| {
    ///         r.get("/users", UsersController::index)
    ///          .post("/users", UsersController::create)
    ///     })
    /// ```
    ///
    /// Génère les routes `/api/v1/users` (GET) et `/api/v1/users` (POST).
    pub fn group(self, prefix: &str, callback: impl FnOnce(Router) -> Router) -> Self {
        let group_router = callback(Router::new());
        Self {
            inner: self.inner.nest(prefix, group_router.inner),
        }
    }

    /// Fusionne un autre `Router` dans celui-ci (sans préfixe).
    ///
    /// Utile pour organiser les routes en plusieurs fichiers :
    /// ```rust,ignore
    /// let router = Router::new()
    ///     .merge(user_routes())
    ///     .merge(post_routes());
    /// ```
    pub fn merge(self, other: Router) -> Self {
        Self {
            inner: self.inner.merge(other.inner),
        }
    }

    /// Convertit en `axum::Router` natif pour le serving.
    ///
    /// Appelé en interne par `HttpServer::new()`.
    pub(crate) fn into_axum(self) -> AxumRouter {
        self.inner
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
