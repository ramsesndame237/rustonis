use anyhow::{bail, Context, Result};
use std::{fs, path::PathBuf};

// ─── Points d'entrée ──────────────────────────────────────────────────────────

/// `rustonis db migrate` — applique les migrations en attente.
pub fn execute_migrate() -> Result<()> {
    let project_root = find_project_root()?;
    let db_url = read_database_url(&project_root)?;
    let migrations_dir = project_root
        .join("database")
        .join("migrations")
        .to_string_lossy()
        .to_string();

    run_async(async move {
        sqlx::any::install_default_drivers();
        let pool = sqlx::AnyPool::connect(&db_url)
            .await
            .with_context(|| format!("Impossible de se connecter à {}", db_url))?;

        let migrator = rustonis_orm::Migrator::new(migrations_dir);
        let count = migrator.run(&pool).await?;

        if count == 0 {
            println!("  ✅ Aucune migration en attente.");
        } else {
            println!("  ✅ {} migration(s) appliquée(s).", count);
        }
        Ok(())
    })
}

/// `rustonis db rollback` — annule le dernier batch de migrations.
pub fn execute_rollback() -> Result<()> {
    let project_root = find_project_root()?;
    let db_url = read_database_url(&project_root)?;
    let migrations_dir = project_root
        .join("database")
        .join("migrations")
        .to_string_lossy()
        .to_string();

    run_async(async move {
        sqlx::any::install_default_drivers();
        let pool = sqlx::AnyPool::connect(&db_url).await?;
        let migrator = rustonis_orm::Migrator::new(migrations_dir);
        let count = migrator.rollback(&pool).await?;

        if count == 0 {
            println!("  ✅ Rien à annuler.");
        } else {
            println!("  ✅ {} migration(s) annulée(s).", count);
        }
        Ok(())
    })
}

/// `rustonis db fresh` — rollback tout + re-run toutes les migrations.
pub fn execute_fresh() -> Result<()> {
    println!("  ⚠️  Cette commande supprime toutes les données !");

    let project_root = find_project_root()?;
    let db_url = read_database_url(&project_root)?;
    let migrations_dir = project_root
        .join("database")
        .join("migrations")
        .to_string_lossy()
        .to_string();

    run_async(async move {
        sqlx::any::install_default_drivers();
        let pool = sqlx::AnyPool::connect(&db_url).await?;
        let migrator = rustonis_orm::Migrator::new(migrations_dir);
        let count = migrator.fresh(&pool).await?;
        println!("  ✅ Base de données recréée, {} migration(s) appliquée(s).", count);
        Ok(())
    })
}

/// `rustonis db seed` — exécute les fichiers dans `database/seeders/`.
pub fn execute_seed() -> Result<()> {
    let project_root = find_project_root()?;
    let db_url = read_database_url(&project_root)?;
    let seeders_dir = project_root.join("database").join("seeders");

    if !seeders_dir.exists() {
        bail!(
            "Aucun dossier de seeds trouvé : {}",
            seeders_dir.display()
        );
    }

    run_async(async move {
        sqlx::any::install_default_drivers();
        let pool = sqlx::AnyPool::connect(&db_url).await?;

        let mut entries: Vec<_> = fs::read_dir(&seeders_dir)
            .with_context(|| format!("Impossible de lire {}", seeders_dir.display()))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().and_then(|x| x.to_str()) == Some("sql")
            })
            .collect();

        entries.sort_by_key(|e| e.file_name());

        let mut count = 0usize;
        for entry in &entries {
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            let sql = fs::read_to_string(&path)
                .with_context(|| format!("Impossible de lire {}", path.display()))?;

            println!("  🌱 Seeding  {}", name);
            sqlx::query::<sqlx::Any>(&sql)
                .execute(&pool)
                .await
                .with_context(|| format!("Erreur dans le seed {}", name))?;
            count += 1;
        }

        println!("  ✅ {} seed(s) exécuté(s).", count);
        Ok(())
    })
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn find_project_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;
    loop {
        let has_cargo = current.join("Cargo.toml").exists();
        let has_env   = current.join(".env").exists()
            || current.join(".env.example").exists();
        if has_cargo && has_env {
            return Ok(current);
        }
        match current.parent() {
            Some(p) => current = p.to_path_buf(),
            None => bail!(
                "Aucun projet Rustonis trouvé.\n\
                 Lance cette commande depuis la racine d'un projet créé avec `rustonis new`."
            ),
        }
    }
}

fn read_database_url(root: &PathBuf) -> Result<String> {
    // Try .env first
    let env_path = root.join(".env");
    if env_path.exists() {
        let content = fs::read_to_string(&env_path)?;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("DATABASE_URL=") {
                return Ok(line["DATABASE_URL=".len()..].trim_matches('"').to_string());
            }
        }
    }
    // Fallback to environment variable
    std::env::var("DATABASE_URL").with_context(|| {
        "DATABASE_URL n'est pas défini dans .env ni dans l'environnement".to_string()
    })
}

fn run_async<F>(fut: F) -> Result<()>
where
    F: std::future::Future<Output = Result<()>>,
{
    tokio::runtime::Runtime::new()
        .context("Impossible de créer le runtime Tokio")?
        .block_on(fut)
}
