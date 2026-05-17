//! refinery-backed migration runner.
//!
//! Each database backend is a separate optional feature — enabling `postgres`
//! never pulls in SQLite code and vice versa.  The URL scheme selects the
//! backend at runtime.
//!
//! Migration files must follow refinery's naming convention:
//! `V{n}__{description}.sql`  (uppercase V, double underscore, .sql suffix).
//!
//! In-memory SQLite is not supported: rusqlite opens a new empty database on
//! every connection, so `run()` and `status()` would see different instances.
//! Use a file path in tests via `tempfile::NamedTempFile`.

use std::collections::HashSet;
use std::fs;

use futures::future::BoxFuture;

use crate::api::migration::{Migration, MigrationStatus};
use crate::api::migration_error::MigrationError;
use crate::api::migration_runner::MigrationRunner;

pub(crate) struct RefineryMigrationRunner {
    database_url:   String,
    migrations_dir: String,
}

impl RefineryMigrationRunner {
    pub(crate) fn new(
        database_url:   impl Into<String>,
        migrations_dir: impl Into<String>,
    ) -> Self {
        Self { database_url: database_url.into(), migrations_dir: migrations_dir.into() }
    }
}

// ── load migration files from disk ────────────────────────────────────────────

fn load_refinery_migrations(dir: &str) -> Result<Vec<refinery::Migration>, MigrationError> {
    let entries = fs::read_dir(dir)
        .map_err(|_| MigrationError::MigrationsDirectoryNotFound(dir.to_string()))?;

    let mut migrations = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|e| MigrationError::MigrationsDirectoryNotFound(e.to_string()))?;
        let filename = entry.file_name().to_string_lossy().to_string();
        if filename.ends_with(".sql") && !filename.ends_with(".down.sql") {
            let sql = fs::read_to_string(entry.path())
                .map_err(|e| MigrationError::MigrationsDirectoryNotFound(e.to_string()))?;
            let stem = filename.trim_end_matches(".sql");
            let m = refinery::Migration::unapplied(stem, &sql)
                .map_err(|e| MigrationError::MigrationsDirectoryNotFound(e.to_string()))?;
            migrations.push(m);
        }
    }
    migrations.sort_by_key(|m| m.version());
    Ok(migrations)
}

// ── MigrationRunner impl ───────────────────────────────────────────────────────

impl MigrationRunner for RefineryMigrationRunner {
    fn run(&self) -> BoxFuture<'_, Result<Vec<Migration>, MigrationError>> {
        Box::pin(async move {
            let url = self.database_url.clone();
            let dir = self.migrations_dir.clone();

            #[cfg(feature = "postgres")]
            if url.starts_with("postgres") {
                return run_postgres(url, dir).await;
            }

            #[cfg(feature = "sqlite")]
            if url.starts_with("sqlite") {
                return run_sqlite(url, dir).await;
            }

            Err(MigrationError::NotConfigured(format!(
                "no compiled-in driver for URL: {url} — enable the `postgres` or `sqlite` feature"
            )))
        })
    }

    fn revert(&self) -> BoxFuture<'_, Result<Migration, MigrationError>> {
        Box::pin(async {
            Err(MigrationError::NotConfigured(
                "revert is not supported by the refinery runner".into(),
            ))
        })
    }

    fn status(&self) -> BoxFuture<'_, Result<Vec<MigrationStatus>, MigrationError>> {
        Box::pin(async move {
            let url = self.database_url.clone();
            let dir = self.migrations_dir.clone();

            #[cfg(feature = "postgres")]
            if url.starts_with("postgres") {
                return status_postgres(url, dir).await;
            }

            #[cfg(feature = "sqlite")]
            if url.starts_with("sqlite") {
                return status_sqlite(url, dir).await;
            }

            Err(MigrationError::NotConfigured(format!(
                "no compiled-in driver for URL: {url} — enable the `postgres` or `sqlite` feature"
            )))
        })
    }
}

// ── postgres backend ───────────────────────────────────────────────────────────

#[cfg(feature = "postgres")]
async fn run_postgres(url: String, dir: String) -> Result<Vec<Migration>, MigrationError> {
    use tokio_postgres::NoTls;

    let migrations = load_refinery_migrations(&dir)?;
    let runner = refinery::Runner::new(&migrations);

    let (mut client, conn) = tokio_postgres::connect(&url, NoTls)
        .await
        .map_err(|e| MigrationError::Connection(e.to_string()))?;
    tokio::spawn(conn);

    let report = runner
        .run_async(&mut client)
        .await
        .map_err(|e| MigrationError::Apply { version: 0, reason: e.to_string() })?;

    let applied = report
        .applied_migrations()
        .iter()
        .map(|m| Migration::applied(m.version() as i64, m.name(), ""))
        .collect();

    Ok(applied)
}

#[cfg(feature = "postgres")]
async fn status_postgres(url: String, dir: String) -> Result<Vec<MigrationStatus>, MigrationError> {
    use tokio_postgres::NoTls;

    let known = load_refinery_migrations(&dir)?;

    let (client, conn) = tokio_postgres::connect(&url, NoTls)
        .await
        .map_err(|e| MigrationError::Connection(e.to_string()))?;
    tokio::spawn(conn);

    let applied: HashSet<i64> = client
        .query("SELECT version FROM refinery_schema_history", &[])
        .await
        .unwrap_or_default()
        .iter()
        .map(|row| row.get::<_, i64>(0))
        .collect();

    Ok(build_statuses(&known, &applied))
}

// ── sqlite backend ─────────────────────────────────────────────────────────────

#[cfg(feature = "sqlite")]
async fn run_sqlite(url: String, dir: String) -> Result<Vec<Migration>, MigrationError> {
    let path = sqlite_path(&url)?;
    tokio::task::spawn_blocking(move || {
        let migrations = load_refinery_migrations(&dir)?;
        let runner = refinery::Runner::new(&migrations);
        let mut conn = rusqlite::Connection::open(&path)
            .map_err(|e| MigrationError::Connection(e.to_string()))?;
        let report = runner
            .run(&mut conn)
            .map_err(|e| MigrationError::Apply { version: 0, reason: e.to_string() })?;
        let applied = report
            .applied_migrations()
            .iter()
            .map(|m| Migration::applied(m.version() as i64, m.name(), ""))
            .collect();
        Ok(applied)
    })
    .await
    .map_err(|e| MigrationError::Internal(e.to_string()))?
}

#[cfg(feature = "sqlite")]
async fn status_sqlite(url: String, dir: String) -> Result<Vec<MigrationStatus>, MigrationError> {
    let path = sqlite_path(&url)?;
    tokio::task::spawn_blocking(move || {
        let known = load_refinery_migrations(&dir)?;
        let conn = rusqlite::Connection::open(&path)
            .map_err(|e| MigrationError::Connection(e.to_string()))?;
        let applied: HashSet<i64> = conn
            .prepare("SELECT version FROM refinery_schema_history")
            .and_then(|mut stmt| {
                stmt.query_map([], |row| row.get::<_, i64>(0))
                    .map(|rows| rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();
        Ok(build_statuses(&known, &applied))
    })
    .await
    .map_err(|e| MigrationError::Internal(e.to_string()))?
}

#[cfg(feature = "sqlite")]
fn sqlite_path(url: &str) -> Result<String, MigrationError> {
    let path = url
        .strip_prefix("sqlite:///")
        .or_else(|| url.strip_prefix("sqlite://"))
        .or_else(|| url.strip_prefix("sqlite:"))
        .unwrap_or(url);

    if path == ":memory:" {
        return Err(MigrationError::Connection(
            "in-memory SQLite is not supported by the refinery runner; use a file path".into(),
        ));
    }
    Ok(path.to_string())
}

// ── shared helper ──────────────────────────────────────────────────────────────

fn build_statuses(
    known:   &[refinery::Migration],
    applied: &HashSet<i64>,
) -> Vec<MigrationStatus> {
    known
        .iter()
        .map(|m| {
            let v = m.version() as i64;
            if applied.contains(&v) {
                MigrationStatus::applied(v, m.name(), "")
            } else {
                MigrationStatus::pending(v, m.name())
            }
        })
        .collect()
}
