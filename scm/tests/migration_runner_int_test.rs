//! Integration tests for the `MigrationRunner` trait contract.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use swe_edge_egress_database_migration::{MigrationError, MigrationRunner, MigrationSvc};

/// @covers: MigrationRunner::run — noop runner returns an empty applied-migrations list
#[tokio::test]
async fn test_migration_runner_run_noop_returns_empty_vec() {
    let runner = MigrationSvc::noop_migration_runner();
    let result = runner.run().await.expect("noop run must succeed");
    assert!(result.is_empty(), "noop runner must apply no migrations");
}

/// @covers: MigrationRunner::status — noop runner reports no migrations
#[tokio::test]
async fn test_migration_runner_status_noop_returns_empty_vec() {
    let runner = MigrationSvc::noop_migration_runner();
    let result = runner.status().await.expect("noop status must succeed");
    assert!(result.is_empty(), "noop runner must report no known migrations");
}

/// @covers: MigrationRunner::revert — noop runner returns NoMigrationToRevert
/// (nothing applied, so nothing to revert)
#[tokio::test]
async fn test_migration_runner_revert_noop_returns_no_migration_to_revert() {
    let runner = MigrationSvc::noop_migration_runner();
    let result = runner.revert().await;
    assert!(
        matches!(result, Err(MigrationError::NoMigrationToRevert)),
        "noop runner revert must return NoMigrationToRevert, got: {result:?}"
    );
}

/// @covers: MigrationRunner — Box<dyn MigrationRunner> implements MigrationRunner via blanket impl
#[tokio::test]
async fn test_migration_runner_boxed_blanket_impl_delegates_run() {
    let runner: Box<dyn MigrationRunner> = MigrationSvc::noop_migration_runner();
    let result = runner.run().await.expect("blanket impl run must succeed");
    assert!(result.is_empty());
}
