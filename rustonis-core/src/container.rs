use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::RwLock;

// Type aliases pour la lisibilité
type ArcAny = Arc<dyn Any + Send + Sync>;
type AsyncFactory = Box<dyn Fn() -> Pin<Box<dyn Future<Output = ArcAny> + Send>> + Send + Sync>;

/// Erreurs possibles lors de la résolution de dépendances.
#[derive(Debug)]
pub enum ContainerError {
    /// Aucun binding trouvé pour ce type
    NotFound(String),
    /// La factory a retourné un type incompatible (ne devrait jamais arriver)
    TypeMismatch(String),
}

impl std::fmt::Display for ContainerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerError::NotFound(t) => write!(f, "No binding found for type: {}", t),
            ContainerError::TypeMismatch(t) => write!(f, "Type mismatch when resolving: {}", t),
        }
    }
}

impl std::error::Error for ContainerError {}

/// Conteneur IoC pour la gestion des dépendances.
///
/// Supporte deux cycles de vie :
/// - **Singleton** : une seule instance partagée (lazy, créée au premier `make()`)
/// - **Transient** : une nouvelle instance à chaque `make()`
///
/// # Exemple
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use rustonis_core::container::Container;
///
/// #[derive(Debug)]
/// struct Logger { prefix: String }
///
/// # #[tokio::main]
/// # async fn main() {
/// let mut container = Container::new();
///
/// container.bind_singleton(|| async {
///     Arc::new(Logger { prefix: "[APP]".to_string() })
/// });
///
/// let logger: Arc<Logger> = container.make().await.unwrap();
/// println!("{:?}", logger);
/// # }
/// ```
pub struct Container {
    /// Cache des singletons déjà construits
    singletons: RwLock<HashMap<TypeId, ArcAny>>,
    /// Factories enregistrées (singletons + transient)
    factories: HashMap<TypeId, AsyncFactory>,
    /// Indique si un binding est singleton (true) ou transient (false)
    lifecycle: HashMap<TypeId, Lifecycle>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Lifecycle {
    Singleton,
    Transient,
}

impl Container {
    /// Crée un nouveau container vide.
    pub fn new() -> Self {
        Self {
            singletons: RwLock::new(HashMap::new()),
            factories: HashMap::new(),
            lifecycle: HashMap::new(),
        }
    }

    /// Enregistre un binding singleton.
    ///
    /// La factory est appelée une seule fois au premier `make()`.
    /// Les appels suivants retournent le même `Arc<T>`.
    pub fn bind_singleton<T, F, Fut>(&mut self, factory: F)
    where
        T: Any + Send + Sync + 'static,
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Arc<T>> + Send + 'static,
    {
        let type_id = TypeId::of::<T>();
        let boxed: AsyncFactory = Box::new(move || {
            let fut = factory();
            Box::pin(async move {
                let instance: Arc<T> = fut.await;
                instance as ArcAny
            })
        });
        self.factories.insert(type_id, boxed);
        self.lifecycle.insert(type_id, Lifecycle::Singleton);
    }

    /// Enregistre un binding transient.
    ///
    /// La factory est appelée à chaque `make()`, produisant une nouvelle instance.
    pub fn bind_transient<T, F, Fut>(&mut self, factory: F)
    where
        T: Any + Send + Sync + 'static,
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Arc<T>> + Send + 'static,
    {
        let type_id = TypeId::of::<T>();
        let boxed: AsyncFactory = Box::new(move || {
            let fut = factory();
            Box::pin(async move {
                let instance: Arc<T> = fut.await;
                instance as ArcAny
            })
        });
        self.factories.insert(type_id, boxed);
        self.lifecycle.insert(type_id, Lifecycle::Transient);
    }

    /// Enregistre une instance existante comme singleton.
    ///
    /// Utile pour injecter des valeurs pré-construites (config, etc.).
    pub fn instance<T>(&mut self, value: Arc<T>)
    where
        T: Any + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let arc_any: ArcAny = value;
        // On enregistre directement dans le cache singleton
        self.singletons.get_mut().insert(type_id, arc_any);
        self.lifecycle.insert(type_id, Lifecycle::Singleton);
    }

    /// Résout une dépendance de type `T`.
    ///
    /// - Singleton : retourne l'instance en cache ou la crée (lazy).
    /// - Transient : crée toujours une nouvelle instance.
    pub async fn make<T>(&self) -> Result<Arc<T>, ContainerError>
    where
        T: Any + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>();

        let lifecycle = self
            .lifecycle
            .get(&type_id)
            .ok_or_else(|| ContainerError::NotFound(type_name.to_string()))?;

        if *lifecycle == Lifecycle::Singleton {
            // Vérifier le cache en lecture d'abord
            {
                let cache = self.singletons.read().await;
                if let Some(existing) = cache.get(&type_id) {
                    return existing
                        .clone()
                        .downcast::<T>()
                        .map_err(|_| ContainerError::TypeMismatch(type_name.to_string()));
                }
            }

            // Pas en cache → construire et mettre en cache
            let factory = self
                .factories
                .get(&type_id)
                .ok_or_else(|| ContainerError::NotFound(type_name.to_string()))?;

            let arc_any = factory().await;
            let arc_t = arc_any
                .clone()
                .downcast::<T>()
                .map_err(|_| ContainerError::TypeMismatch(type_name.to_string()))?;

            let mut cache = self.singletons.write().await;
            cache.insert(type_id, arc_any);

            Ok(arc_t)
        } else {
            // Transient : pas de cache
            let factory = self
                .factories
                .get(&type_id)
                .ok_or_else(|| ContainerError::NotFound(type_name.to_string()))?;

            let arc_any = factory().await;
            arc_any
                .downcast::<T>()
                .map_err(|_| ContainerError::TypeMismatch(type_name.to_string()))
        }
    }

    /// Vérifie qu'un binding existe pour `T`.
    pub fn has<T: Any + 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.lifecycle.contains_key(&type_id) || {
            // Peut aussi être enregistré via `instance()` directement dans les singletons
            // (accès synchrone au cache via get_mut n'est pas possible ici,
            //  mais lifecycle est toujours inséré dans instance())
            false
        }
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[derive(Debug, PartialEq)]
    struct Database {
        url: String,
    }

    #[derive(Debug)]
    struct CacheService {
        ttl: u64,
    }

    // Compteur global pour vérifier combien de fois une factory est appelée
    static DB_FACTORY_CALLS: AtomicU32 = AtomicU32::new(0);

    #[tokio::test]
    async fn test_singleton_is_resolved_correctly() {
        let mut container = Container::new();
        container.bind_singleton(|| async {
            Arc::new(Database {
                url: "postgres://localhost/test".to_string(),
            })
        });

        let db: Arc<Database> = container.make().await.unwrap();
        assert_eq!(db.url, "postgres://localhost/test");
    }

    #[tokio::test]
    async fn test_singleton_returns_same_instance() {
        DB_FACTORY_CALLS.store(0, Ordering::SeqCst);

        let mut container = Container::new();
        container.bind_singleton(|| async {
            DB_FACTORY_CALLS.fetch_add(1, Ordering::SeqCst);
            Arc::new(Database {
                url: "postgres://localhost/test".to_string(),
            })
        });

        let db1: Arc<Database> = container.make().await.unwrap();
        let db2: Arc<Database> = container.make().await.unwrap();

        // Même pointeur Arc
        assert!(Arc::ptr_eq(&db1, &db2));
        // Factory appelée UNE seule fois
        assert_eq!(DB_FACTORY_CALLS.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_transient_returns_different_instances() {
        let mut container = Container::new();
        container.bind_transient(|| async {
            Arc::new(CacheService { ttl: 300 })
        });

        let c1: Arc<CacheService> = container.make().await.unwrap();
        let c2: Arc<CacheService> = container.make().await.unwrap();

        // Pointeurs différents
        assert!(!Arc::ptr_eq(&c1, &c2));
    }

    #[tokio::test]
    async fn test_make_returns_not_found_for_unregistered_type() {
        let container = Container::new();
        let result: Result<Arc<Database>, ContainerError> = container.make().await;

        assert!(result.is_err());
        matches!(result.unwrap_err(), ContainerError::NotFound(_));
    }

    #[tokio::test]
    async fn test_has_returns_true_when_registered() {
        let mut container = Container::new();
        container.bind_singleton(|| async {
            Arc::new(Database {
                url: "test".to_string(),
            })
        });

        assert!(container.has::<Database>());
        assert!(!container.has::<CacheService>());
    }

    #[tokio::test]
    async fn test_instance_registers_existing_arc() {
        let mut container = Container::new();
        let db = Arc::new(Database {
            url: "postgres://production/db".to_string(),
        });
        container.instance(db.clone());

        let resolved: Arc<Database> = container.make().await.unwrap();
        assert!(Arc::ptr_eq(&db, &resolved));
    }

    #[tokio::test]
    async fn test_multiple_types_coexist() {
        let mut container = Container::new();

        container.bind_singleton(|| async {
            Arc::new(Database {
                url: "postgres://localhost/test".to_string(),
            })
        });
        container.bind_transient(|| async {
            Arc::new(CacheService { ttl: 60 })
        });

        let db: Arc<Database> = container.make().await.unwrap();
        let cache: Arc<CacheService> = container.make().await.unwrap();

        assert_eq!(db.url, "postgres://localhost/test");
        assert_eq!(cache.ttl, 60);
    }
}
