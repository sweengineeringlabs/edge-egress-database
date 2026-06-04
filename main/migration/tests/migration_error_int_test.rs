//! Tests for `MigrationError` display and variant coverage.

use swe_edge_egress_database_migration::MigrationError;

#[test]
fn test_migration_error_connection_display_includes_message() {
    let e = MigrationError::Connection("refused".into());
    assert!(e.to_string().contains("refused"));
}

#[test]
fn test_migration_error_apply_display_includes_version_and_source() {
    let e = MigrationError::Apply {
        version: 3,
        reason: "syntax error".into(),
    };
    let s = e.to_string();
    assert!(s.contains('3'));
    assert!(s.contains("syntax error"));
}

#[test]
fn test_migration_error_no_migration_to_revert_display() {
    let e = MigrationError::NoMigrationToRevert;
    assert!(e.to_string().contains("no applied"));
}

#[test]
fn test_migration_error_revert_display_includes_version() {
    let e = MigrationError::Revert {
        version: 5,
        reason: "constraint".into(),
    };
    let s = e.to_string();
    assert!(s.contains('5'));
    assert!(s.contains("constraint"));
}

#[test]
fn test_migration_error_not_configured_display() {
    let e = MigrationError::NotConfigured("no driver".into());
    assert!(e.to_string().contains("no driver"));
}

#[test]
fn test_migration_error_internal_display() {
    let e = MigrationError::Internal("task panicked".into());
    assert!(e.to_string().contains("task panicked"));
}

#[test]
fn test_migration_error_migrations_directory_not_found_display() {
    let e = MigrationError::MigrationsDirectoryNotFound("/nonexistent".into());
    assert!(e.to_string().contains("/nonexistent"));
}
