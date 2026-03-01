use std::{
    fs,
    path::Path,
};

use sqlx::AnyPool;

use crate::error::OrmError;

const MIGRATIONS_TABLE: &str = "_rustonis_migrations";

/// One migration file (up + optional down SQL).
#[derive(Debug, Clone)]
pub struct Migration {
    /// Timestamp prefix, e.g. `20260301120000`.
    pub timestamp: String,
    /// Human-readable name, e.g. `create_users_table`.
    pub name: String,
    /// SQL to apply.
    pub up_sql: String,
    /// SQL to rollback (everything after `-- Down` in the file).
    pub down_sql: Option<String>,
}

impl Migration {
    /// Canonical file name: `{timestamp}_{name}.sql`
    pub fn file_name(&self) -> String {
        format!("{}_{}.sql", self.timestamp, self.name)
    }
}

/// Runs file-based migrations from `database/migrations/`.
///
/// Each file is named `YYYYMMDDHHMMSS_description.sql` and may contain a
/// `-- Down` separator to separate the up and down SQL blocks.
pub struct Migrator {
    migrations_dir: String,
}

impl Migrator {
    pub fn new(migrations_dir: impl Into<String>) -> Self {
        Self { migrations_dir: migrations_dir.into() }
    }

    /// Default: `./database/migrations`
    pub fn default_dir() -> Self {
        Self::new("database/migrations")
    }

    // ── SETUP ─────────────────────────────────────────────────────────────

    async fn ensure_table(&self, pool: &AnyPool) -> Result<(), OrmError> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                name       TEXT    NOT NULL UNIQUE,
                batch      INTEGER NOT NULL,
                run_at     TEXT    NOT NULL
            )",
            MIGRATIONS_TABLE
        );
        sqlx::query::<sqlx::Any>(&sql)
            .execute(pool)
            .await
            .map_err(OrmError::Query)?;
        Ok(())
    }

    async fn applied_names(&self, pool: &AnyPool) -> Result<Vec<String>, OrmError> {
        let sql = format!("SELECT name FROM {} ORDER BY id ASC", MIGRATIONS_TABLE);
        let rows: Vec<(String,)> = sqlx::query_as::<sqlx::Any, (String,)>(&sql)
            .fetch_all(pool)
            .await
            .map_err(OrmError::Query)?;
        Ok(rows.into_iter().map(|(n,)| n).collect())
    }

    async fn current_batch(&self, pool: &AnyPool) -> Result<i64, OrmError> {
        let sql = format!(
            "SELECT COALESCE(MAX(batch), 0) FROM {}",
            MIGRATIONS_TABLE
        );
        let (n,): (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(&sql)
            .fetch_one(pool)
            .await
            .map_err(OrmError::Query)?;
        Ok(n)
    }

    // ── FILE LOADING ──────────────────────────────────────────────────────

    pub fn load_files(&self) -> Result<Vec<Migration>, OrmError> {
        let dir = Path::new(&self.migrations_dir);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut migrations = Vec::new();
        let mut entries: Vec<_> = fs::read_dir(dir)
            .map_err(|e| OrmError::Migration(e.to_string()))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().and_then(|x| x.to_str()) == Some("sql")
            })
            .collect();

        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();

            let parts: Vec<&str> = stem.splitn(2, '_').collect();
            if parts.len() != 2 {
                return Err(OrmError::Migration(format!(
                    "Invalid migration file name '{}'. Expected TIMESTAMP_name.sql",
                    stem
                )));
            }

            let content = fs::read_to_string(&path)
                .map_err(|e| OrmError::Migration(e.to_string()))?;

            let (up_sql, down_sql) = split_migration(&content);

            migrations.push(Migration {
                timestamp: parts[0].to_string(),
                name:      stem.clone(),
                up_sql,
                down_sql,
            });
        }

        Ok(migrations)
    }

    // ── COMMANDS ──────────────────────────────────────────────────────────

    /// Run all pending migrations.  Returns the number applied.
    pub async fn run(&self, pool: &AnyPool) -> Result<usize, OrmError> {
        self.ensure_table(pool).await?;
        let applied = self.applied_names(pool).await?;
        let batch   = self.current_batch(pool).await? + 1;
        let files   = self.load_files()?;

        let pending: Vec<_> = files
            .iter()
            .filter(|m| !applied.contains(&m.name))
            .collect();

        let count = pending.len();
        for m in &pending {
            println!("  ↑ Migrating  {}", m.name);
            sqlx::query::<sqlx::Any>(&m.up_sql)
                .execute(pool)
                .await
                .map_err(|e| OrmError::Migration(format!("{}: {}", m.name, e)))?;

            let sql = format!(
                "INSERT INTO {} (name, batch, run_at) VALUES (?, ?, datetime('now'))",
                MIGRATIONS_TABLE
            );
            sqlx::query::<sqlx::Any>(&sql)
                .bind(&m.name)
                .bind(batch)
                .execute(pool)
                .await
                .map_err(OrmError::Query)?;
        }

        Ok(count)
    }

    /// Rollback the last batch of migrations.  Returns the number rolled back.
    pub async fn rollback(&self, pool: &AnyPool) -> Result<usize, OrmError> {
        self.ensure_table(pool).await?;
        let batch = self.current_batch(pool).await?;
        if batch == 0 {
            return Ok(0);
        }

        let sql = format!(
            "SELECT name FROM {} WHERE batch = ? ORDER BY id DESC",
            MIGRATIONS_TABLE
        );
        let rows: Vec<(String,)> = sqlx::query_as::<sqlx::Any, (String,)>(&sql)
            .bind(batch)
            .fetch_all(pool)
            .await
            .map_err(OrmError::Query)?;

        let files = self.load_files()?;
        let count = rows.len();

        for (name,) in &rows {
            let migration = files.iter().find(|m| &m.name == name);
            if let Some(m) = migration {
                if let Some(down) = &m.down_sql {
                    println!("  ↓ Rolling back  {}", name);
                    sqlx::query::<sqlx::Any>(down)
                        .execute(pool)
                        .await
                        .map_err(|e| OrmError::Migration(format!("{}: {}", name, e)))?;
                } else {
                    return Err(OrmError::Migration(format!(
                        "Migration '{}' has no -- Down block.",
                        name
                    )));
                }
            }

            let del = format!("DELETE FROM {} WHERE name = ?", MIGRATIONS_TABLE);
            sqlx::query::<sqlx::Any>(&del)
                .bind(name)
                .execute(pool)
                .await
                .map_err(OrmError::Query)?;
        }

        Ok(count)
    }

    /// Drop all tables and re-run every migration (development only).
    pub async fn fresh(&self, pool: &AnyPool) -> Result<usize, OrmError> {
        // Rollback all batches
        loop {
            let n = self.rollback(pool).await?;
            if n == 0 { break; }
        }
        self.run(pool).await
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn split_migration(content: &str) -> (String, Option<String>) {
    if let Some(idx) = content.find("-- Down") {
        let up   = content[..idx].trim().to_string();
        let down = content[idx + "-- Down".len()..].trim().to_string();
        (up, if down.is_empty() { None } else { Some(down) })
    } else {
        (content.trim().to_string(), None)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_no_down() {
        let (up, down) = split_migration("CREATE TABLE users (id INTEGER);");
        assert_eq!(up, "CREATE TABLE users (id INTEGER);");
        assert!(down.is_none());
    }

    #[test]
    fn test_split_with_down() {
        let content = "CREATE TABLE users (id INTEGER);\n-- Down\nDROP TABLE users;";
        let (up, down) = split_migration(content);
        assert_eq!(up, "CREATE TABLE users (id INTEGER);");
        assert_eq!(down.unwrap(), "DROP TABLE users;");
    }
}
