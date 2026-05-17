//! SAF layer — public factory surface for migration runners.

pub use crate::api::{Migration, MigrationError, MigrationRunner, MigrationStatus};
use crate::core::noop_migration_runner::NoopMigrationRunner;

/// Returns a no-op [`MigrationRunner`] that always succeeds without touching
/// any database.
///
/// Use in tests or in services that manage migrations externally and only
/// need a concrete runner in the type signature.
pub fn noop_migration_runner() -> impl MigrationRunner {
    NoopMigrationRunner
}

/// Returns a PostgreSQL [`MigrationRunner`] backed by sqlx.
///
/// Migrations are loaded from `migrations_dir` at runtime — sqlx naming
/// convention: `V{version}__{description}.sql` and optionally
/// `V{version}__{description}.down.sql` for reversible migrations.
///
/// # Errors
///
/// Returns [`MigrationError::MigrationsDirectoryNotFound`] if `migrations_dir`
/// does not exist or is not readable.
#[cfg(feature = "postgres")]
pub fn postgres_migration_runner(
    database_url: impl Into<String>,
    migrations_dir: impl Into<String>,
) -> impl MigrationRunner {
    crate::core::sqlx_migration_runner::SqlxMigrationRunner::new(database_url, migrations_dir)
}

/// Returns a SQLite [`MigrationRunner`] backed by sqlx.
///
/// Same semantics as [`postgres_migration_runner`].
#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
pub fn sqlite_migration_runner(
    database_url: impl Into<String>,
    migrations_dir: impl Into<String>,
) -> impl MigrationRunner {
    crate::core::sqlx_migration_runner::SqlxMigrationRunner::new(database_url, migrations_dir)
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
    fn test_noop_migration_runner_is_returnable_as_dyn_trait() {
        fn accept(_: &dyn MigrationRunner) {}
        let r = noop_migration_runner();
        accept(&r);
    }
}
