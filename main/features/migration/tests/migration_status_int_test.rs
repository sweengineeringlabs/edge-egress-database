//! Tests for `MigrationStatus` value object.

use swe_edge_egress_database_migration::MigrationStatus;

/// @covers: pending
#[test]
fn test_migration_status_struct_pending_sets_applied_false() {
    let s = MigrationStatus::pending(1, "init");
    assert!(!s.applied, "pending status must have applied=false");
    assert_eq!(s.migration.version, 1);
    assert!(s.migration.applied_at.is_none());
}

/// @covers: applied
#[test]
fn test_migration_status_struct_applied_sets_applied_true() {
    let s = MigrationStatus::applied(2, "add_index", "2026-01-01T00:00:00Z");
    assert!(s.applied, "applied status must have applied=true");
    assert_eq!(s.migration.version, 2);
    assert!(s.migration.applied_at.is_some());
}

#[test]
fn test_migration_status_struct_description_propagated_to_migration() {
    let s = MigrationStatus::pending(3, "create_table");
    assert_eq!(s.migration.description, "create_table");
}
