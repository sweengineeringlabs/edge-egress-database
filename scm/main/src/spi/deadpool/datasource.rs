//! `deadpool`-backed datasource: build a configured pool and verify connectivity.
//!
//! Migrations are NOT run here — that is `core::refinery`'s job. This module
//! only builds the `deadpool` pool and does a single ping to confirm the DB
//! is reachable. All external-library types are confined to this `spi/` module
//! per ADR-008.

use crate::api::error::MigrationError;
use crate::api::types::{DatabaseConfig, DriverKind};
use crate::spi::deadpool::db_pool::DbPool;

/// Namespace for deadpool pool construction operations.
pub(crate) struct DeadpoolDatasource;

impl DeadpoolDatasource {
    /// Build a pool for `cfg` and ping it to confirm reachability.
    ///
    /// Does NOT run migrations; call `MigrationSvc::connect_and_migrate` for
    /// the full migrate-then-pool flow.
    pub(crate) async fn connect(cfg: &DatabaseConfig) -> Result<DbPool, MigrationError> {
        match cfg.driver {
            #[cfg(feature = "sqlite")]
            DriverKind::Sqlite => {
                let pool = Self::build_sqlite_pool(cfg)?;
                // Ping: open one connection to confirm the file is accessible.
                let _ = pool
                    .get()
                    .await
                    .map_err(|e| MigrationError::Connection(e.to_string()))?;
                Ok(DbPool::Sqlite(pool))
            }
            #[cfg(feature = "postgres")]
            DriverKind::Postgres => {
                let pool = Self::build_pg_pool(cfg)?;
                // Ping: attempt one connection to confirm the server is reachable.
                let _ = pool
                    .get()
                    .await
                    .map_err(|e| MigrationError::Connection(e.to_string()))?;
                Ok(DbPool::Postgres(pool))
            }
            #[allow(unreachable_patterns)]
            other => Err(MigrationError::NotConfigured(format!(
                "driver {other:?} is selected but its cargo feature is not enabled \
                 — build with --features {}",
                other.url_scheme().trim_end_matches(':')
            ))),
        }
    }

    // ── pool builders (sync) ────────────────────────────────────────────────

    #[cfg(feature = "sqlite")]
    fn build_sqlite_pool(cfg: &DatabaseConfig) -> Result<deadpool_sqlite::Pool, MigrationError> {
        let path = Self::sqlite_file_path(&cfg.url)?;
        let mut pool_cfg = deadpool_sqlite::Config::new(path);
        pool_cfg.pool = Some(deadpool_sqlite::PoolConfig {
            max_size: cfg.max_connections as usize,
            timeouts: deadpool_sqlite::Timeouts {
                wait: Some(std::time::Duration::from_secs(cfg.acquire_timeout_secs)),
                ..Default::default()
            },
            ..Default::default()
        });
        pool_cfg
            .create_pool(deadpool_sqlite::Runtime::Tokio1)
            .map_err(|e| MigrationError::Connection(e.to_string()))
    }

    #[cfg(feature = "postgres")]
    fn build_pg_pool(cfg: &DatabaseConfig) -> Result<deadpool_postgres::Pool, MigrationError> {
        let mut pg_cfg = deadpool_postgres::Config::new();
        pg_cfg.url = Some(cfg.url.clone());
        pg_cfg.pool = Some(deadpool_postgres::PoolConfig {
            max_size: cfg.max_connections as usize,
            timeouts: deadpool_postgres::Timeouts {
                wait: Some(std::time::Duration::from_secs(cfg.acquire_timeout_secs)),
                ..Default::default()
            },
            ..Default::default()
        });
        pg_cfg
            .create_pool(
                Some(deadpool_postgres::Runtime::Tokio1),
                tokio_postgres::NoTls,
            )
            .map_err(|e| MigrationError::Connection(e.to_string()))
    }

    // ── URL helpers ─────────────────────────────────────────────────────────

    /// Strip the `sqlite:` URL scheme prefix; return the bare file path.
    ///
    /// deadpool-sqlite's `Config::new` takes a file-system path, not a URL.
    #[cfg(feature = "sqlite")]
    fn sqlite_file_path(url: &str) -> Result<String, MigrationError> {
        let path = url
            .strip_prefix("sqlite:///")
            .or_else(|| url.strip_prefix("sqlite://"))
            .or_else(|| url.strip_prefix("sqlite:"))
            .unwrap_or(url);

        if path == ":memory:" {
            return Err(MigrationError::Connection(
                "in-memory SQLite is not supported; use a file path \
                 (refinery and deadpool-sqlite both require persistent storage)"
                    .into(),
            ));
        }
        Ok(path.to_string())
    }
}
