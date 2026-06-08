//! SAF factory methods on [`MigrationSvc`] — the public datasource + migration surface.

use crate::api::traits::MigrationRunner;
use crate::api::types::MigrationSvc;
use crate::core::noop::NoopMigrationRunner;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
use crate::api::error::MigrationError;
#[cfg(any(feature = "postgres", feature = "sqlite"))]
use crate::api::types::DatabaseConfig;
#[cfg(any(feature = "postgres", feature = "sqlite"))]
use crate::spi::deadpool::DbPool;

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
    /// Migrations are applied by `refinery` on a dedicated connection before the
    /// pool is opened. Running `connect_and_migrate` twice applies nothing the
    /// second time — refinery is idempotent. Consumers query through the concrete
    /// deadpool pool ([`DbPool::as_sqlite`] / [`DbPool::as_postgres`]) to back
    /// their `edge_domain::Repository` adapters.
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
        let dir = cfg.migrations_dir.as_deref().ok_or_else(|| {
            MigrationError::NotConfigured(
                "connect_and_migrate requires `migrations_dir` in [database]; \
                 use connect() to open a pool without migrating"
                    .into(),
            )
        })?;
        // 1. Run migrations on a fresh connection (refinery, RUSTSEC-clean).
        let runner = crate::core::refinery::RefineryMigrationRunner::new(cfg.url.as_str(), dir);
        runner.run().await?;
        // 2. Open the deadpool (ping included).
        crate::spi::deadpool::DeadpoolDatasource::connect(cfg).await
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
        crate::spi::deadpool::DeadpoolDatasource::connect(cfg).await
    }

    /// Open a [`MigrationRunner`] bound to `cfg`'s URL and migrations directory.
    ///
    /// Unlike [`connect_and_migrate`](Self::connect_and_migrate), this does not
    /// apply migrations eagerly — the caller drives `run()` / `status()` on the
    /// returned runner (e.g. to inspect status before applying). The runner
    /// uses a fresh connection per call; call [`connect`](Self::connect) separately
    /// to obtain a pool for consumer queries.
    ///
    /// # Errors
    ///
    /// - [`MigrationError::NotConfigured`] if `cfg.migrations_dir` is `None`, or
    ///   the driver's cargo feature is off.
    #[cfg(any(feature = "postgres", feature = "sqlite"))]
    pub async fn migration_runner(
        cfg: &DatabaseConfig,
    ) -> Result<Box<dyn MigrationRunner>, MigrationError> {
        let dir = cfg.migrations_dir.clone().ok_or_else(|| {
            MigrationError::NotConfigured(
                "migration_runner requires `migrations_dir` in [database]".into(),
            )
        })?;
        Ok(Box::new(
            crate::core::refinery::RefineryMigrationRunner::new(cfg.url.as_str(), dir),
        ))
    }
}
