pub mod database;
pub mod error;
pub mod migration;
pub mod model;
pub mod query_builder;
pub mod value;

pub use database::Database;
pub use error::OrmError;
pub use migration::Migrator;
pub use model::Model;
pub use query_builder::QueryBuilder;
pub use value::OrmValue;

/// Re-export the `#[model]` derive macro.
pub use rustonis_macros::model;

pub mod prelude {
    pub use crate::{Database, Migrator, Model, OrmError, OrmValue, QueryBuilder};
    pub use crate::model;
    pub use sqlx::AnyPool;
}
