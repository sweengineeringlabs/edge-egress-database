//! Value object for a single database migration.

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
