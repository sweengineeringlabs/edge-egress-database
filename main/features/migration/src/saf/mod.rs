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

/// Connect to a database and return a [`MigrationRunner`] backed by sqlx.
///
/// The database backend is selected at runtime from the URL scheme —
/// no recompilation needed to switch from SQLite to PostgreSQL:
///
/// | URL scheme         | Backend    | Feature required |
/// |--------------------|------------|-----------------|
/// | `postgres://…`     | PostgreSQL | `postgres`      |
/// | `sqlite://…`       | SQLite     | `sqlite`        |
/// | `sqlite::memory:`  | SQLite in-memory | `sqlite`  |
/// | `mysql://…`        | MySQL      | `mysql`         |
///
/// Multiple features may be active simultaneously; the URL determines
/// which compiled-in driver handles the connection.
///
/// # Errors
///
/// Returns [`MigrationError::Connection`] if the database is unreachable,
/// or [`MigrationError::MigrationsDirectoryNotFound`] if `migrations_dir`
/// does not exist when `run()` / `status()` / `revert()` is first called.
///
/// # Example
///
/// ```rust,no_run
/// # #[cfg(feature = "sqlite")]
/// # async fn example() -> Result<(), swe_edge_egress_database_migration::MigrationError> {
/// use swe_edge_egress_database_migration::{migration_runner, MigrationRunner};
///
/// let runner = migration_runner("sqlite::memory:", "./migrations").await?;
/// let applied = runner.run().await?;
/// println!("applied {} migration(s)", applied.len());
/// # Ok(())
/// # }
/// ```
#[cfg(any(feature = "postgres", feature = "sqlite", feature = "mysql"))]
pub async fn migration_runner(
    database_url: impl Into<String>,
    migrations_dir: impl Into<String>,
) -> Result<impl MigrationRunner, MigrationError> {
    crate::core::sqlx_migration_runner::SqlxMigrationRunner::connect(database_url, migrations_dir)
        .await
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
