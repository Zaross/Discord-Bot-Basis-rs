use anyhow::{bail, Context};
use sqlx::{any::AnyPoolOptions, AnyPool};
use std::path::Path;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Sqlite,
    Postgres,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Backend::Sqlite => write!(f, "SQLite"),
            Backend::Postgres => write!(f, "Postgres"),
        }
    }
}

#[derive(Clone)]
pub struct Database {
    pub pool: AnyPool,
    pub backend: Backend,
}

impl Database {
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        sqlx::any::install_default_drivers();

        let backend = detect_backend(url)?;
        info!("Connecting to {} database…", backend);

        let pool = AnyPoolOptions::new()
            .max_connections(match backend {
                Backend::Sqlite => 1,
                Backend::Postgres => 10,
            })
            .connect(url)
            .await
            .with_context(|| format!("Failed to connect to {} at {}", backend, url))?;

        info!("Connected to {}.", backend);

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .context("Failed to run database migrations")?;

        info!("Migrations applied.");

        Ok(Self { pool, backend })
    }
}

pub fn resolve_database_url(raw: Option<&str>, bot_folder: &Path) -> anyhow::Result<String> {
    match raw {
        Some(url) if url.starts_with("postgres") => Ok(url.to_string()),

        Some(url) if url.starts_with("sqlite://") => {
            ensure_sqlite_dir(url)?;
            Ok(append_rwc(url))
        }

        None | Some("sqlite") | Some("") => {
            let bot_name = bot_folder
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("bot");

            let db_dir = bot_folder.join("Database");
            std::fs::create_dir_all(&db_dir).with_context(|| {
                format!("Failed to create database directory '{}'", db_dir.display())
            })?;

            let db_path = db_dir
                .join(format!("{}.db", bot_name))
                .display()
                .to_string()
                .replace('\\', "/");

            Ok(format!("sqlite://{}?mode=rwc", db_path))
        }

        Some(other) => bail!(
            "Unrecognised DATABASE_URL '{}'. \
             Use 'sqlite', 'sqlite://<path>', or 'postgres://<host>/<db>'",
            other
        ),
    }
}

fn append_rwc(url: &str) -> String {
    if url.contains('?') {
        if url.contains("mode=") {
            url.to_string()
        } else {
            format!("{}&mode=rwc", url)
        }
    } else {
        format!("{}?mode=rwc", url)
    }
}

fn ensure_sqlite_dir(url: &str) -> anyhow::Result<()> {
    let path_part = url
        .trim_start_matches("sqlite://")
        .split('?')
        .next()
        .unwrap_or("");

    if path_part.is_empty() {
        return Ok(());
    }

    let path = std::path::Path::new(path_part);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create directory '{}'", parent.display())
            })?;
        }
    }
    Ok(())
}

fn detect_backend(url: &str) -> anyhow::Result<Backend> {
    if url.starts_with("sqlite://") {
        Ok(Backend::Sqlite)
    } else if url.starts_with("postgres://") || url.starts_with("postgresql://") {
        Ok(Backend::Postgres)
    } else {
        bail!("Cannot determine database backend from URL '{}'", url)
    }
}
