//! Value objects for migration state.

use serde::{Deserialize, Serialize};

/// Represents a single database migration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Migration {
    /// Monotonically increasing version number (from the filename prefix).
    pub version: i64,
    /// Human-readable description (from the filename).
    pub description: String,
    /// ISO-8601 timestamp when the migration was applied. `None` if pending.
    pub applied_at: Option<String>,
}

impl Migration {
    /// Construct a pending migration (not yet applied).
    pub fn pending(version: i64, description: impl Into<String>) -> Self {
        Self { version, description: description.into(), applied_at: None }
    }

    /// Construct an applied migration.
    pub fn applied(version: i64, description: impl Into<String>, at: impl Into<String>) -> Self {
        Self { version, description: description.into(), applied_at: Some(at.into()) }
    }
}

/// The status of a single migration as seen by the runner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationStatus {
    /// The migration this status entry describes.
    pub migration: Migration,
    /// `true` when the migration has been applied to the target database.
    pub applied: bool,
}

impl MigrationStatus {
    /// Construct a status entry for a migration that has not yet been applied.
    pub fn pending(version: i64, description: impl Into<String>) -> Self {
        Self { migration: Migration::pending(version, description), applied: false }
    }

    /// Construct a status entry for a migration that has already been applied.
    pub fn applied(version: i64, description: impl Into<String>, at: impl Into<String>) -> Self {
        Self { migration: Migration::applied(version, description, at), applied: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_pending_has_no_applied_at() {
        let m = Migration::pending(1, "create users");
        assert_eq!(m.version, 1);
        assert!(m.applied_at.is_none());
    }

    #[test]
    fn test_migration_applied_stores_timestamp() {
        let m = Migration::applied(2, "add index", "2026-05-17T00:00:00Z");
        assert!(m.applied_at.is_some());
    }

    #[test]
    fn test_migration_status_pending_is_not_applied() {
        let s = MigrationStatus::pending(1, "init");
        assert!(!s.applied);
    }

    #[test]
    fn test_migration_status_applied_is_applied() {
        let s = MigrationStatus::applied(1, "init", "2026-05-17T00:00:00Z");
        assert!(s.applied);
    }
}
