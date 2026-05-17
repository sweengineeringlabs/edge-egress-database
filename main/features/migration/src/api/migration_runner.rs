//! `MigrationRunner` — public trait for database schema migration.

use futures::future::BoxFuture;

use crate::api::migration::{Migration, MigrationStatus};
use crate::api::migration_error::MigrationError;

/// Applies and reverts database migrations.
///
/// Consumers call the SAF factory that matches their database backend
/// (`postgres_migration_runner`, `sqlite_migration_runner`, etc.) and receive
/// `impl MigrationRunner`.  The concrete runner type stays `pub(crate)` in
/// `core/`; callers never name it.
///
/// # Example
///
/// ```rust,ignore
/// let runner = swe_edge_egress_database_migration::postgres_migration_runner(
///     "postgres://localhost/mydb",
///     "./migrations",
/// ).await?;
///
/// let applied = runner.run().await?;
/// println!("Applied {} migration(s)", applied.len());
/// ```
pub trait MigrationRunner: Send + Sync {
    /// Apply all pending migrations in version order.
    ///
    /// Returns the list of migrations that were applied in this call.
    /// Returns an empty `Vec` if all migrations are already applied.
    fn run(&self) -> BoxFuture<'_, Result<Vec<Migration>, MigrationError>>;

    /// Revert the most recently applied migration.
    ///
    /// Returns the migration that was reverted.
    /// Returns [`MigrationError::NoMigrationToRevert`] if none are applied.
    /// Returns [`MigrationError::NotConfigured`] for runners that do not
    /// support reversible migrations.
    fn revert(&self) -> BoxFuture<'_, Result<Migration, MigrationError>>;

    /// Return the status of every known migration.
    ///
    /// Each entry reports whether the migration has been applied to the
    /// target database.  Useful for health checks and deployment validation.
    fn status(&self) -> BoxFuture<'_, Result<Vec<MigrationStatus>, MigrationError>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_runner_is_object_safe() {
        fn _assert(_: &dyn MigrationRunner) {}
    }
}
