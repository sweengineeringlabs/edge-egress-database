//! SAF factory methods on [`MigrationSvc`] — the public datasource + migration surface.

use crate::api::traits::MigrationRunner;
use crate::api::types::MigrationSvc;
use crate::core::noop::NoopMigrationRunner;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
use crate::api::error::MigrationError;
#[cfg(any(feature = "postgres", feature = "sqlite"))]
use crate::api::types::DatabaseConfig;
#[cfg(any(feature = "postgres", feature = "sqlite"))]
use crate::spi::sqlx::DbPool;

impl MigrationSvc {
    /// Return a config builder pre-seeded with this crate's package name and version.
    pub fn create_config_builder() -> swe_edge_configbuilder::ConfigBuilderImpl {
        let mut b = swe_edge_configbuilder::ConfigBuilderImpl::new();
        b = b.with_name(env!("CARGO_PKG_NAME"));
        b = b.with_version(env!("CARGO_PKG_VERSION"));
        b
    }

    /// Returns a no-op [`MigrationRunner`] that always succeeds without touching
    /// any database.
    ///
    /// Use in tests or in services that manage migrations externally and only
    /// need a concrete `MigrationRunner` in the type signature.
    pub fn noop_migration_runner() -> Box<dyn MigrationRunner> {
        Box::new(NoopMigrationRunner)
    }

    /// Open a connection pool for `cfg` and run all pending migrations, returning
    /// the ready [`DbPool`].
    ///
    /// The returned pool has had every migration in `cfg.migrations_dir` applied
    /// idempotently — running twice applies nothing the second time. Consumers
    /// query through the concrete `sqlx` pool ([`DbPool::as_sqlite`] /
    /// [`DbPool::as_postgres`]) to back their `edge_domain::Repository` adapters.
    ///
    /// # Errors
    ///
    /// - [`MigrationError::NotConfigured`] if `cfg.migrations_dir` is `None`, or
    ///   the selected driver's cargo feature is not enabled.
    /// - [`MigrationError::Connection`] if the database is unreachable.
    /// - [`MigrationError::MigrationsDirectoryNotFound`] if the directory is absent.
    /// - [`MigrationError::Apply`] if a migration fails to apply.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "sqlite")]
    /// # async fn example() -> Result<(), swe_edge_egress_database_migration::MigrationError> {
    /// use swe_edge_egress_database_migration::{DatabaseConfig, DriverKind, MigrationSvc};
    ///
    /// let cfg = DatabaseConfig {
    ///     driver: DriverKind::Sqlite,
    ///     url: "sqlite:///./obsrv.db".into(),
    ///     max_connections: 5,
    ///     acquire_timeout_secs: 30,
    ///     idle_timeout_secs: None,
    ///     migrations_dir: Some("./migrations".into()),
    /// };
    /// let pool = MigrationSvc::connect_and_migrate(&cfg).await?;
    /// # let _ = pool;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(any(feature = "postgres", feature = "sqlite"))]
    pub async fn connect_and_migrate(cfg: &DatabaseConfig) -> Result<DbPool, MigrationError> {
        crate::spi::sqlx::datasource::connect_and_migrate(cfg).await
    }

    /// Open a connection pool for `cfg` **without** running migrations.
    ///
    /// Use when migrations are managed separately, or to obtain a pool for a
    /// driver whose schema is already current.
    ///
    /// # Errors
    ///
    /// - [`MigrationError::NotConfigured`] if the driver's cargo feature is off.
    /// - [`MigrationError::Connection`] if the database is unreachable.
    #[cfg(any(feature = "postgres", feature = "sqlite"))]
    pub async fn connect(cfg: &DatabaseConfig) -> Result<DbPool, MigrationError> {
        crate::spi::sqlx::datasource::connect(cfg).await
    }

    /// Open a pool for `cfg` and return a [`MigrationRunner`] bound to it.
    ///
    /// Unlike [`connect_and_migrate`](Self::connect_and_migrate), this does not
    /// apply migrations eagerly — the caller drives `run()` / `status()` on the
    /// returned runner (e.g. to inspect status before applying).
    ///
    /// # Errors
    ///
    /// - [`MigrationError::NotConfigured`] if `cfg.migrations_dir` is `None`, or
    ///   the driver's cargo feature is off.
    /// - [`MigrationError::Connection`] if the database is unreachable.
    #[cfg(any(feature = "postgres", feature = "sqlite"))]
    pub async fn migration_runner(
        cfg: &DatabaseConfig,
    ) -> Result<Box<dyn MigrationRunner>, MigrationError> {
        let dir = cfg.migrations_dir.clone().ok_or_else(|| {
            MigrationError::NotConfigured(
                "migration_runner requires `migrations_dir` in [database]".into(),
            )
        })?;
        let pool = crate::spi::sqlx::datasource::connect(cfg).await?;
        Ok(Box::new(crate::spi::sqlx::SqlxMigrationRunner::new(
            pool, dir,
        )))
    }
}
