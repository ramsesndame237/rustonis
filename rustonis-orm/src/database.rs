use sqlx::AnyPool;

use crate::error::OrmError;

/// Wraps a SQLx `AnyPool`.
///
/// Supports SQLite (default), PostgreSQL and MySQL via Cargo feature flags.
/// The connection URL determines the driver at runtime:
///
/// - `sqlite://./database/dev.db`
/// - `postgres://user:pass@localhost/mydb`
/// - `mysql://user:pass@localhost/mydb`
#[derive(Clone)]
pub struct Database {
    pool: AnyPool,
}

impl Database {
    /// Connect to the database specified by `url`.
    ///
    /// Registers all compiled-in drivers automatically.
    pub async fn connect(url: &str) -> Result<Self, OrmError> {
        sqlx::any::install_default_drivers();
        let pool = AnyPool::connect(url)
            .await
            .map_err(OrmError::Connection)?;
        Ok(Self { pool })
    }

    /// Connect using `DATABASE_URL` from the environment.
    pub async fn connect_env() -> Result<Self, OrmError> {
        let url = std::env::var("DATABASE_URL").map_err(|_| {
            OrmError::Connection(sqlx::Error::Configuration(
                "DATABASE_URL not set".into(),
            ))
        })?;
        Self::connect(&url).await
    }

    /// Raw pool — use when you need to call SQLx directly.
    pub fn pool(&self) -> &AnyPool {
        &self.pool
    }
}
