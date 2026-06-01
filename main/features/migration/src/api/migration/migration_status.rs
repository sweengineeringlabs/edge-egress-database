//! Value object for the status of a single migration.

use serde::{Deserialize, Serialize};

use super::migration::Migration;

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
        Self {
            migration: Migration::pending(version, description),
            applied: false,
        }
    }

    /// Construct a status entry for a migration that has already been applied.
    pub fn applied(version: i64, description: impl Into<String>, at: impl Into<String>) -> Self {
        Self {
            migration: Migration::applied(version, description, at),
            applied: true,
        }
    }
}
