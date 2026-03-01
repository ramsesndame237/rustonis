use std::sync::Arc;

use async_trait::async_trait;

use crate::container::Container;

/// Trait que chaque provider du framework doit implémenter.
///
/// Le cycle de vie est en deux phases :
///
/// 1. **`register`** : enregistre les bindings dans le container.
///    Le container n'est pas encore buildé — on ne peut pas appeler `make()` ici.
///
/// 2. **`boot`** : s'exécute après que tous les providers ont été enregistrés.
///    Le container est finalisé — on peut résoudre des dépendances.
///
/// # Exemple
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use async_trait::async_trait;
/// use rustonis_core::container::Container;
/// use rustonis_core::provider::ServiceProvider;
///
/// struct DatabaseService { url: String }
///
/// struct DatabaseProvider;
///
/// #[async_trait]
/// impl ServiceProvider for DatabaseProvider {
///     async fn register(&self, container: &mut Container) {
///         container.bind_singleton(|| async {
///             Arc::new(DatabaseService {
///                 url: std::env::var("DATABASE_URL")
///                     .unwrap_or_else(|_| "sqlite::memory:".to_string()),
///             })
///         });
///     }
///
///     async fn boot(&self, container: &Arc<Container>) {
///         // Warm up : résoudre la dépendance pour vérifier la connexion
///         let _db: Arc<DatabaseService> = container.make().await.unwrap();
///     }
/// }
/// ```
#[async_trait]
pub trait ServiceProvider: Send + Sync {
    /// Phase 1 : enregistrer les bindings dans le container.
    async fn register(&self, container: &mut Container);

    /// Phase 2 : démarrer le provider (warmup, validation de config, etc.).
    ///
    /// Implémentation par défaut : no-op.
    async fn boot(&self, _container: &Arc<Container>) {}
}
