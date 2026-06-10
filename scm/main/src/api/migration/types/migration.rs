//! Value object for a single database migration.

use serde::{Deserialize, Serialize};

/// Represents a single database migration.
///
/// Returned by [`MigrationRunner::run`](crate::MigrationRunner::run) and
/// [`MigrationRunner::status`](crate::MigrationRunner::status). Construct
/// directly only in unit tests via [`Migration::pending`] and
/// [`Migration::applied`].
///
/// # Examples
///
/// ```rust
/// use swe_edge_egress_database_migration::Migration;
///
/// let pending = Migration::pending(1, "create users table");
/// assert_eq!(pending.version, 1);
/// assert_eq!(pending.description, "create users table");
/// assert!(pending.applied_at.is_none());
///
/// let applied = Migration::applied(2, "add email index", "2026-06-04T09:00:00Z");
/// assert_eq!(applied.version, 2);
/// assert!(applied.applied_at.is_some());
/// assert_ne!(pending, applied);
/// ```
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
        Self {
            version,
            description: description.into(),
            applied_at: None,
        }
    }

    /// Construct an applied migration.
    pub fn applied(version: i64, description: impl Into<String>, at: impl Into<String>) -> Self {
        Self {
            version,
            description: description.into(),
            applied_at: Some(at.into()),
        }
    }
}
