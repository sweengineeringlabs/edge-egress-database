//! Tests for `MigrationSvc` — the public factory handle type.

use swe_edge_egress_database_migration::MigrationSvc;

#[test]
fn test_migration_svc_is_constructible() {
    let _svc = MigrationSvc;
}

#[test]
fn test_migration_svc_create_config_builder_is_callable() {
    let _b = MigrationSvc::create_config_builder();
}
