//! Tests for `ApplicationConfigBuilder`.

use swe_edge_egress_database_migration::ApplicationConfigBuilder;

/// @covers: ApplicationConfigBuilder — type alias is usable as ConfigBuilderImpl.
#[test]
fn test_application_config_builder_is_constructible() {
    let _builder: ApplicationConfigBuilder = swe_edge_configbuilder::ConfigBuilderImpl::new();
}

/// @covers: ApplicationConfigBuilder — create_config_builder seeds name and version.
#[test]
fn test_application_config_builder_via_migration_svc_factory() {
    use swe_edge_egress_database_migration::MigrationSvc;
    let _b = MigrationSvc::create_config_builder();
}
