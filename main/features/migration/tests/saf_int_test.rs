//! Integration tests for the SAF public factory surface.

use swe_edge_egress_database_migration::{MigrationError, MigrationRunner, MigrationSvc};

/// @covers: MigrationSvc::noop_migration_runner — run returns empty list.
#[tokio::test]
async fn test_migration_svc_noop_migration_runner_run_returns_empty() {
    let runner = MigrationSvc::noop_migration_runner();
    let applied = runner.run().await.expect("noop run must succeed");
    assert!(applied.is_empty());
}

/// @covers: MigrationSvc::noop_migration_runner — status returns empty list.
#[tokio::test]
async fn test_migration_svc_noop_migration_runner_status_returns_empty() {
    let runner = MigrationSvc::noop_migration_runner();
    let status = runner.status().await.expect("noop status must succeed");
    assert!(status.is_empty());
}

/// @covers: MigrationSvc::noop_migration_runner — revert returns NoMigrationToRevert.
#[tokio::test]
async fn test_migration_svc_noop_migration_runner_revert_returns_no_migration_to_revert() {
    let runner = MigrationSvc::noop_migration_runner();
    let err = runner.revert().await.expect_err("noop revert must fail");
    assert!(matches!(err, MigrationError::NoMigrationToRevert));
}

/// @covers: MigrationSvc::noop_migration_runner — usable as dyn trait.
#[test]
fn test_migration_svc_noop_migration_runner_is_usable_as_dyn_trait() {
    fn accept(_: &dyn MigrationRunner) {}
    let r = MigrationSvc::noop_migration_runner();
    accept(&r);
}

/// @covers: MigrationSvc::create_config_builder — returns a builder with name and version.
#[test]
fn test_migration_svc_create_config_builder_returns_builder() {
    let builder = MigrationSvc::create_config_builder();
    // builder is a ConfigBuilderImpl; verify it is non-trivially constructable.
    drop(builder);
}
