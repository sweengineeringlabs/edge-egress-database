//! `sqlx`-backed datasource: build a configured pool and run migrations.
//!
//! All `sqlx` usage is confined to this `spi/` module (ADR-008). The neutral
//! `api/` surface (`DatabaseConfig`, `MigrationRunner`, `MigrationError`) names
//! no `sqlx` type. Functions are associated methods on [`SqlxDatasource`] per
//! the SEA OOP rule (no free-standing functions in layer files).

use std::collections::HashSet;
use std::path::Path;

use crate::api::error::MigrationError;
use crate::api::migration::{Migration, MigrationStatus};
use crate::api::types::{DatabaseConfig, DriverKind};
use crate::spi::sqlx::db_pool::DbPool;

/// Namespace for the `sqlx`-backed connect/migrate operations.
pub(crate) struct SqlxDatasource;

impl SqlxDatasource {
    /// Open a connection pool for `cfg` without running migrations.
    pub(crate) async fn connect(cfg: &DatabaseConfig) -> Result<DbPool, MigrationError> {
        match cfg.driver {
            #[cfg(feature = "sqlite")]
            DriverKind::Sqlite => Ok(DbPool::Sqlite(Self::build_sqlite_pool(cfg).await?)),
            #[cfg(feature = "postgres")]
            DriverKind::Postgres => Ok(DbPool::Postgres(Self::build_pg_pool(cfg).await?)),
            #[allow(unreachable_patterns)]
            other => Err(MigrationError::NotConfigured(format!(
                "driver {other:?} is selected but its cargo feature is not enabled \
                 — build with --features {}",
                other.url_scheme().trim_end_matches(':')
            ))),
        }
    }

    /// Open a pool for `cfg`, run pending migrations from `cfg.migrations_dir`,
    /// and return the ready pool.
    pub(crate) async fn connect_and_migrate(
        cfg: &DatabaseConfig,
    ) -> Result<DbPool, MigrationError> {
        let dir = cfg.migrations_dir.as_deref().ok_or_else(|| {
            MigrationError::NotConfigured(
                "connect_and_migrate requires `migrations_dir` in [database]; \
                 use connect() to open a pool without migrating"
                    .into(),
            )
        })?;
        let pool = Self::connect(cfg).await?;
        Self::run_migrations(&pool, dir).await?;
        Ok(pool)
    }

    /// Apply all pending migrations in `dir`; return the ones newly applied.
    pub(crate) async fn run_migrations(
        pool: &DbPool,
        dir: &str,
    ) -> Result<Vec<Migration>, MigrationError> {
        let migrator = Self::load_migrator(dir).await?;
        let before = Self::applied_versions(pool).await;

        match pool {
            #[cfg(feature = "sqlite")]
            DbPool::Sqlite(p) => migrator.run(p).await.map_err(Self::to_apply_error)?,
            #[cfg(feature = "postgres")]
            DbPool::Postgres(p) => migrator.run(p).await.map_err(Self::to_apply_error)?,
        };

        let after = Self::applied_versions(pool).await;
        let newly = migrator
            .iter()
            .filter(|m| after.contains(&m.version) && !before.contains(&m.version))
            .map(|m| Migration::applied(m.version, m.description.as_ref(), ""))
            .collect();
        Ok(newly)
    }

    /// Report the applied/pending status of every migration in `dir`.
    pub(crate) async fn migration_status(
        pool: &DbPool,
        dir: &str,
    ) -> Result<Vec<MigrationStatus>, MigrationError> {
        let migrator = Self::load_migrator(dir).await?;
        let applied = Self::applied_versions(pool).await;
        Ok(migrator
            .iter()
            .map(|m| {
                if applied.contains(&m.version) {
                    MigrationStatus::applied(m.version, m.description.as_ref(), "")
                } else {
                    MigrationStatus::pending(m.version, m.description.as_ref())
                }
            })
            .collect())
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    async fn load_migrator(dir: &str) -> Result<::sqlx::migrate::Migrator, MigrationError> {
        if !Path::new(dir).is_dir() {
            return Err(MigrationError::MigrationsDirectoryNotFound(dir.to_string()));
        }
        ::sqlx::migrate::Migrator::new(Path::new(dir))
            .await
            .map_err(|e| MigrationError::Apply {
                version: 0,
                reason: format!("failed to load migrations from {dir}: {e}"),
            })
    }

    /// Versions recorded in `_sqlx_migrations`. Absent table (no migration has
    /// run yet) yields an empty set rather than an error.
    async fn applied_versions(pool: &DbPool) -> HashSet<i64> {
        const QUERY: &str = "SELECT version FROM _sqlx_migrations";
        let versions: Vec<i64> = match pool {
            #[cfg(feature = "sqlite")]
            DbPool::Sqlite(p) => ::sqlx::query_scalar(QUERY)
                .fetch_all(p)
                .await
                .unwrap_or_default(),
            #[cfg(feature = "postgres")]
            DbPool::Postgres(p) => ::sqlx::query_scalar(QUERY)
                .fetch_all(p)
                .await
                .unwrap_or_default(),
        };
        versions.into_iter().collect()
    }

    fn to_apply_error(e: ::sqlx::migrate::MigrateError) -> MigrationError {
        MigrationError::Apply {
            version: 0,
            reason: e.to_string(),
        }
    }

    #[cfg(feature = "sqlite")]
    async fn build_sqlite_pool(cfg: &DatabaseConfig) -> Result<::sqlx::SqlitePool, MigrationError> {
        use std::str::FromStr;
        use std::time::Duration;

        let options = ::sqlx::sqlite::SqliteConnectOptions::from_str(&cfg.url)
            .map_err(|e| MigrationError::Connection(e.to_string()))?
            .create_if_missing(true);

        let mut pool_options = ::sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(cfg.max_connections)
            .acquire_timeout(Duration::from_secs(cfg.acquire_timeout_secs));
        if let Some(idle) = cfg.idle_timeout_secs {
            pool_options = pool_options.idle_timeout(Duration::from_secs(idle));
        }

        pool_options
            .connect_with(options)
            .await
            .map_err(|e| MigrationError::Connection(e.to_string()))
    }

    #[cfg(feature = "postgres")]
    async fn build_pg_pool(cfg: &DatabaseConfig) -> Result<::sqlx::PgPool, MigrationError> {
        use std::time::Duration;

        let mut pool_options = ::sqlx::postgres::PgPoolOptions::new()
            .max_connections(cfg.max_connections)
            .acquire_timeout(Duration::from_secs(cfg.acquire_timeout_secs));
        if let Some(idle) = cfg.idle_timeout_secs {
            pool_options = pool_options.idle_timeout(Duration::from_secs(idle));
        }

        pool_options
            .connect(&cfg.url)
            .await
            .map_err(|e| MigrationError::Connection(e.to_string()))
    }
}
