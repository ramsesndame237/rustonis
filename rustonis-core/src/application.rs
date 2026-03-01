use std::sync::Arc;

use crate::container::Container;
use crate::provider::ServiceProvider;

/// Erreurs levées lors du boot de l'application.
#[derive(Debug)]
pub enum BootError {
    /// Un provider a rencontré une erreur pendant le boot
    ProviderFailed { provider: String, reason: String },
}

impl std::fmt::Display for BootError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootError::ProviderFailed { provider, reason } => {
                write!(f, "Provider '{}' failed to boot: {}", provider, reason)
            }
        }
    }
}

impl std::error::Error for BootError {}

/// Point d'entrée principal de Rustonis.
///
/// `Application` orchestre les `ServiceProvider` en deux phases :
///
/// 1. **Register** : tous les providers enregistrent leurs bindings.
/// 2. **Boot** : le container est finalisé (`Arc<Container>`), puis
///    chaque provider reçoit le container pour initialiser ses services.
///
/// # Exemple
///
/// ```rust,ignore
/// use rustonis_core::application::Application;
///
/// let container = Application::new()
///     .register(AppProvider)
///     .register(DatabaseProvider)
///     .boot()
///     .await
///     .expect("Application boot failed");
///
/// // container est maintenant un Arc<Container> résolvable
/// ```
pub struct Application {
    providers: Vec<Box<dyn ServiceProvider>>,
}

impl Application {
    /// Crée une nouvelle application sans providers.
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Ajoute un `ServiceProvider` à l'application.
    ///
    /// Consomme `self` pour permettre le chaînage fluent :
    /// ```rust,ignore
    /// Application::new()
    ///     .register(ProviderA)
    ///     .register(ProviderB);
    /// ```
    pub fn register<P: ServiceProvider + 'static>(mut self, provider: P) -> Self {
        self.providers.push(Box::new(provider));
        self
    }

    /// Démarre l'application.
    ///
    /// **Phase 1 — Register** : chaque provider appelle `container.bind_*()`.
    /// **Phase 2 — Boot** : le container est scellé dans un `Arc`,
    ///   chaque provider reçoit la référence pour ses initialisations.
    ///
    /// Retourne l'`Arc<Container>` prêt à l'emploi.
    pub async fn boot(self) -> Result<Arc<Container>, BootError> {
        let mut container = Container::new();

        // Phase 1 : tous les providers enregistrent leurs bindings
        for provider in &self.providers {
            provider.register(&mut container).await;
        }

        // Container finalisé — on passe à Arc pour le partager
        let container = Arc::new(container);

        // Phase 2 : tous les providers bootent avec le container complet
        for provider in &self.providers {
            provider.boot(&container).await;
        }

        Ok(container)
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct AppConfig {
        name: String,
    }

    struct CacheService {
        max_size: usize,
    }

    // Flags globaux pour vérifier que les phases sont bien appelées
    static REGISTER_CALLED: AtomicBool = AtomicBool::new(false);
    static BOOT_CALLED: AtomicBool = AtomicBool::new(false);

    struct AppProvider;

    #[async_trait]
    impl ServiceProvider for AppProvider {
        async fn register(&self, container: &mut Container) {
            REGISTER_CALLED.store(true, Ordering::SeqCst);
            container.bind_singleton(|| async {
                Arc::new(AppConfig {
                    name: "rustonis-test".to_string(),
                })
            });
        }

        async fn boot(&self, _container: &Arc<Container>) {
            BOOT_CALLED.store(true, Ordering::SeqCst);
        }
    }

    struct CacheProvider;

    #[async_trait]
    impl ServiceProvider for CacheProvider {
        async fn register(&self, container: &mut Container) {
            container.bind_transient(|| async {
                Arc::new(CacheService { max_size: 1024 })
            });
        }
    }

    #[tokio::test]
    async fn test_application_boot_calls_register_and_boot() {
        REGISTER_CALLED.store(false, Ordering::SeqCst);
        BOOT_CALLED.store(false, Ordering::SeqCst);

        let container = Application::new()
            .register(AppProvider)
            .boot()
            .await
            .unwrap();

        assert!(REGISTER_CALLED.load(Ordering::SeqCst), "register() should have been called");
        assert!(BOOT_CALLED.load(Ordering::SeqCst), "boot() should have been called");
        // Container fonctionnel
        let config: Arc<AppConfig> = container.make().await.unwrap();
        assert_eq!(config.name, "rustonis-test");
    }

    #[tokio::test]
    async fn test_application_with_multiple_providers() {
        let container = Application::new()
            .register(AppProvider)
            .register(CacheProvider)
            .boot()
            .await
            .unwrap();

        // Les deux bindings sont disponibles
        let config: Arc<AppConfig> = container.make().await.unwrap();
        let cache: Arc<CacheService> = container.make().await.unwrap();

        assert_eq!(config.name, "rustonis-test");
        assert_eq!(cache.max_size, 1024);
    }

    #[tokio::test]
    async fn test_empty_application_boots_successfully() {
        let container = Application::new().boot().await;
        assert!(container.is_ok());
    }

    #[tokio::test]
    async fn test_providers_receive_full_container_in_boot_phase() {
        // Provider qui résout une dépendance pendant boot()
        struct BootValidatorProvider;

        static BOOT_RESOLVED: AtomicBool = AtomicBool::new(false);

        #[async_trait]
        impl ServiceProvider for BootValidatorProvider {
            async fn register(&self, container: &mut Container) {
                container.bind_singleton(|| async {
                    Arc::new(AppConfig {
                        name: "boot-test".to_string(),
                    })
                });
            }

            async fn boot(&self, container: &Arc<Container>) {
                // Doit pouvoir résoudre ce que d'autres providers ont enregistré
                let config: Arc<AppConfig> = container.make().await.unwrap();
                assert_eq!(config.name, "boot-test");
                BOOT_RESOLVED.store(true, Ordering::SeqCst);
            }
        }

        BOOT_RESOLVED.store(false, Ordering::SeqCst);

        Application::new()
            .register(BootValidatorProvider)
            .boot()
            .await
            .unwrap();

        assert!(BOOT_RESOLVED.load(Ordering::SeqCst));
    }
}
