use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

pub fn execute(watch: bool, port: Option<u16>) -> Result<()> {
    if !is_rustonis_project() {
        bail!(
            "Aucun projet Rustonis trouvé dans le répertoire courant.\n\
             Lancez 'rustonis new <nom>' pour créer un nouveau projet."
        );
    }

    if watch {
        run_with_watch(port)
    } else {
        run_dev_server(port)
    }
}

fn is_rustonis_project() -> bool {
    Path::new("Cargo.toml").exists()
        && (Path::new(".env").exists() || Path::new(".env.example").exists())
}

fn run_dev_server(port: Option<u16>) -> Result<()> {
    println!("🦀 Démarrage du serveur de développement Rustonis...");
    println!("   Ctrl+C pour arrêter");
    println!();

    let mut cmd = Command::new("cargo");
    cmd.arg("run");

    if let Some(p) = port {
        cmd.env("APP_PORT", p.to_string());
    }

    let status = cmd.status()?;

    if !status.success() {
        bail!("Le serveur s'est arrêté avec une erreur");
    }

    Ok(())
}

fn run_with_watch(port: Option<u16>) -> Result<()> {
    if !cargo_watch_available() {
        bail!(
            "cargo-watch n'est pas installé.\n\
             Installez-le avec : cargo install cargo-watch\n\
             Puis relancez   : rustonis serve --watch"
        );
    }

    println!("🦀 Démarrage du serveur Rustonis avec hot reload...");
    println!("   En attente de modifications...");
    println!("   Ctrl+C pour arrêter");
    println!();

    let mut cmd = Command::new("cargo");
    cmd.args(["watch", "-x", "run", "-q"]);

    if let Some(p) = port {
        cmd.env("APP_PORT", p.to_string());
    }

    let status = cmd.status()?;

    if !status.success() {
        bail!("Le serveur s'est arrêté avec une erreur");
    }

    Ok(())
}

fn cargo_watch_available() -> bool {
    Command::new("cargo-watch")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
