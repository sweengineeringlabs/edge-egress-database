//! No-op migration runner — always succeeds, never applies or reverts anything.
//!
//! Use in tests and in services that manage migrations externally (e.g. via
//! a CI/CD pipeline tool) and just need a `MigrationRunner` in the type
//! signature without triggering real schema changes.

use futures::future::BoxFuture;

use crate::api::migration::{Migration, MigrationStatus};
use crate::api::migration_error::MigrationError;
use crate::api::migration_runner::MigrationRunner;

pub(crate) struct NoopMigrationRunner;

impl MigrationRunner for NoopMigrationRunner {
    fn run(&self) -> BoxFuture<'_, Result<Vec<Migration>, MigrationError>> {
        Box::pin(async { Ok(vec![]) })
    }

    fn revert(&self) -> BoxFuture<'_, Result<Migration, MigrationError>> {
        Box::pin(async { Err(MigrationError::NoMigrationToRevert) })
    }

    fn status(&self) -> BoxFuture<'_, Result<Vec<MigrationStatus>, MigrationError>> {
        Box::pin(async { Ok(vec![]) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_returns_empty_vec() {
        let r = NoopMigrationRunner;
        let applied = r.run().await.expect("noop run must succeed");
        assert!(applied.is_empty(), "noop runner must apply no migrations");
    }

    #[tokio::test]
    async fn test_status_returns_empty_vec() {
        let r = NoopMigrationRunner;
        let statuses = r.status().await.expect("noop status must succeed");
        assert!(statuses.is_empty(), "noop runner has no known migrations");
    }

    #[tokio::test]
    async fn test_revert_returns_no_migration_to_revert() {
        let r = NoopMigrationRunner;
        let err = r.revert().await.expect_err("noop revert must fail");
        assert!(
            matches!(err, MigrationError::NoMigrationToRevert),
            "expected NoMigrationToRevert, got {err}",
        );
    }
}
