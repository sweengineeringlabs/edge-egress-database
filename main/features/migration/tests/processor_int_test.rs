//! Tests for the `Processor` trait contract and its implementation on `MigrationSvc`.

use swe_edge_egress_database_migration::{MigrationSvc, Processor};

#[test]
fn test_processor_trait_migration_svc_describe_returns_label() {
    let svc = MigrationSvc;
    assert_eq!(svc.describe(), "database-migration");
}

#[test]
fn test_processor_trait_migration_svc_is_send_sync() {
    fn _assert_send_sync<T: Send + Sync>() {}
    _assert_send_sync::<MigrationSvc>();
}
