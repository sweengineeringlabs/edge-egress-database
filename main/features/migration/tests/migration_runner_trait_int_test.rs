//! Tests for the `MigrationRunner` trait contract.

use swe_edge_egress_database_migration::MigrationRunner;

#[test]
fn test_migration_runner_trait_is_object_safe() {
    fn _assert(_: &dyn MigrationRunner) {}
}

#[test]
fn test_migration_runner_trait_can_be_stored_as_arc() {
    use std::sync::Arc;
    use swe_edge_egress_database_migration::MigrationSvc;
    let runner: Arc<dyn MigrationRunner> = Arc::new(MigrationSvc::noop_migration_runner());
    drop(runner);
}
