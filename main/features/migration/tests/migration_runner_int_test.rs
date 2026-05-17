//! Integration tests for the migration runner public API.

use swe_edge_egress_database_migration::{noop_migration_runner, MigrationError, MigrationRunner};

/// @covers: noop_migration_runner — run returns empty list when no migrations are configured.
#[tokio::test]
async fn test_noop_migration_runner_run_returns_empty_list() {
    let runner = noop_migration_runner();
    let applied = runner.run().await.expect("noop run must succeed");
    assert!(
        applied.is_empty(),
        "noop runner must report zero applied migrations, got {}",
        applied.len(),
    );
}

/// @covers: noop_migration_runner — status returns empty list.
#[tokio::test]
async fn test_noop_migration_runner_status_returns_empty_list() {
    let runner = noop_migration_runner();
    let statuses = runner.status().await.expect("noop status must succeed");
    assert!(
        statuses.is_empty(),
        "noop runner has no known migrations, got {}",
        statuses.len(),
    );
}

/// @covers: noop_migration_runner — revert returns NoMigrationToRevert.
#[tokio::test]
async fn test_noop_migration_runner_revert_returns_no_migration_to_revert_error() {
    let runner = noop_migration_runner();
    let err = runner
        .revert()
        .await
        .expect_err("noop revert must return an error when nothing is applied");
    assert!(
        matches!(err, MigrationError::NoMigrationToRevert),
        "expected NoMigrationToRevert, got: {err}",
    );
}

/// @covers: MigrationRunner trait — object-safe; can be stored as Arc<dyn>.
#[test]
fn test_migration_runner_can_be_stored_as_arc_dyn_trait() {
    use std::sync::Arc;
    let runner: Arc<dyn MigrationRunner> = Arc::new(noop_migration_runner());
    // Verify the Arc compiles; no assertion needed — compile-time proof.
    drop(runner);
}

/// @covers: noop_migration_runner — run is idempotent.
#[tokio::test]
async fn test_noop_migration_runner_run_is_idempotent() {
    let runner = noop_migration_runner();
    let first = runner.run().await.expect("first run must succeed");
    let second = runner.run().await.expect("second run must succeed");
    assert_eq!(first.len(), second.len(), "noop run must be idempotent");
}
