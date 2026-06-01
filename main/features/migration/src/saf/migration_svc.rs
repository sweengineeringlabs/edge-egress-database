//! SAF factory methods on [`MigrationSvc`] for assembling [`MigrationRunner`] instances.

#[cfg(any(feature = "postgres", feature = "sqlite"))]
use crate::api::error::MigrationError;
use crate::api::traits::MigrationRunner;
use crate::api::types::MigrationSvc;
use crate::core::noop::NoopMigrationRunner;

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

    /// Connect to a database and return a [`MigrationRunner`] backed by refinery.
    ///
    /// The backend is selected at runtime from the URL scheme:
    ///
    /// | URL scheme       | Backend    | Feature required |
    /// |------------------|------------|-----------------|
    /// | `postgres://…`   | PostgreSQL | `postgres`      |
    /// | `sqlite:///…`    | SQLite     | `sqlite`        |
    ///
    /// Migration files in `migrations_dir` must follow refinery's naming
    /// convention: `V{n}__{description}.sql` (uppercase V, double underscore).
    ///
    /// In-memory SQLite (`sqlite::memory:`) is not supported — each rusqlite
    /// connection opens a fresh database, so state would not persist between
    /// `run()` and `status()` calls.  Use a file path instead, e.g. via
    /// `tempfile::NamedTempFile` in tests.
    ///
    /// # Errors
    ///
    /// Returns [`MigrationError::Connection`] if the database is unreachable, or
    /// [`MigrationError::MigrationsDirectoryNotFound`] if `migrations_dir` does
    /// not exist when `run()` / `status()` is called.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "sqlite")]
    /// # async fn example() -> Result<(), swe_edge_egress_database_migration::MigrationError> {
    /// use swe_edge_egress_database_migration::{MigrationRunner, MigrationSvc};
    ///
    /// let runner = MigrationSvc::migration_runner("sqlite:///./dev.db", "./migrations").await?;
    /// let applied = runner.run().await?;
    /// println!("applied {} migration(s)", applied.len());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(any(feature = "postgres", feature = "sqlite"))]
    pub async fn migration_runner(
        database_url: impl Into<String>,
        migrations_dir: impl Into<String>,
    ) -> Result<Box<dyn MigrationRunner>, MigrationError> {
        Ok(Box::new(
            crate::core::refinery::RefineryMigrationRunner::new(database_url, migrations_dir),
        ))
    }
}
