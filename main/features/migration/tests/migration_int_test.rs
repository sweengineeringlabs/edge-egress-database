//! Tests for `Migration` and `MigrationStatus` value objects.

use swe_edge_egress_database_migration::{Migration, MigrationStatus};

/// @covers: pending
#[test]
fn test_migration_struct_pending_has_no_applied_at() {
    let m = Migration::pending(1, "create users");
    assert_eq!(m.version, 1);
    assert!(m.applied_at.is_none());
}

/// @covers: applied
#[test]
fn test_migration_struct_applied_stores_timestamp() {
    let m = Migration::applied(2, "add index", "2026-05-17T00:00:00Z");
    assert!(m.applied_at.is_some());
    assert_eq!(m.applied_at.as_deref(), Some("2026-05-17T00:00:00Z"));
}

/// @covers: pending
#[test]
fn test_migration_status_struct_pending_is_not_applied() {
    let s = MigrationStatus::pending(1, "init");
    assert!(!s.applied);
    assert_eq!(s.migration.version, 1);
}

/// @covers: applied
#[test]
fn test_migration_status_struct_applied_is_applied() {
    let s = MigrationStatus::applied(1, "init", "2026-05-17T00:00:00Z");
    assert!(s.applied);
    assert_eq!(s.migration.version, 1);
}

#[test]
fn test_migration_struct_description_stored_correctly() {
    let m = Migration::pending(3, "add_email_column");
    assert_eq!(m.description, "add_email_column");
}
