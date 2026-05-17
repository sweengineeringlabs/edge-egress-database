//! SAF layer — public factory surface for migration runners.

pub use crate::api::{Migration, MigrationError, MigrationRunner, MigrationStatus};
use crate::core::noop_migration_runner::NoopMigrationRunner;

/// Returns a no-op [`MigrationRunner`] that always succeeds without touching
/// any database.
///
/// Use in tests or in services that manage migrations externally and only
/// need a concrete `MigrationRunner` in the type signature.
pub fn noop_migration_runner() -> impl MigrationRunner {
    NoopMigrationRunner
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
/// use swe_edge_egress_database_migration::{migration_runner, MigrationRunner};
///
/// let runner = migration_runner("sqlite:///./dev.db", "./migrations").await?;
/// let applied = runner.run().await?;
/// println!("applied {} migration(s)", applied.len());
/// # Ok(())
/// # }
/// ```
#[cfg(any(feature = "postgres", feature = "sqlite"))]
pub async fn migration_runner(
    database_url: impl Into<String>,
    migrations_dir: impl Into<String>,
) -> Result<impl MigrationRunner, MigrationError> {
    Ok(
        crate::core::refinery_migration_runner::RefineryMigrationRunner::new(
            database_url,
            migrations_dir,
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_migration_runner_run_returns_empty() {
        let runner = noop_migration_runner();
        let applied = runner.run().await.expect("noop run must succeed");
        assert!(applied.is_empty());
    }

    #[tokio::test]
    async fn test_noop_migration_runner_status_returns_empty() {
        let runner = noop_migration_runner();
        let status = runner.status().await.expect("noop status must succeed");
        assert!(status.is_empty());
    }

    #[tokio::test]
    async fn test_noop_migration_runner_revert_returns_no_migration_to_revert() {
        let runner = noop_migration_runner();
        let err = runner.revert().await.expect_err("noop revert must fail");
        assert!(matches!(err, MigrationError::NoMigrationToRevert));
    }

    #[test]
    fn test_noop_migration_runner_is_usable_as_dyn_trait() {
        fn accept(_: &dyn MigrationRunner) {}
        let r = noop_migration_runner();
        accept(&r);
    }
}
