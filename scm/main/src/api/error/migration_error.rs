//! Error types for the migration runner.

use thiserror::Error;

/// Errors that can occur during migration operations.
#[derive(Debug, Error)]
pub enum MigrationError {
    /// Could not connect to the database.
    #[error("connection failed: {0}")]
    Connection(String),

    /// A migration failed to apply.
    #[error("failed to apply migration v{version}: {reason}")]
    Apply {
        /// The version number of the migration that failed.
        version: i64,
        /// The underlying error message from the database driver.
        reason: String,
    },

    /// A migration failed to revert.
    #[error("failed to revert migration v{version}: {reason}")]
    Revert {
        /// The version number of the migration that failed to revert.
        version: i64,
        /// The underlying error message from the database driver.
        reason: String,
    },

    /// The migrations directory does not exist or is not readable.
    #[error("migrations directory not found or not readable: {0}")]
    MigrationsDirectoryNotFound(String),

    /// `revert` was called but there are no applied migrations to revert.
    #[error("no applied migration to revert")]
    NoMigrationToRevert,

    /// The migration runner is not configured (e.g. no feature flag enabled).
    #[error("migration runner not configured: {0}")]
    NotConfigured(String),

    /// An unexpected internal runtime error (e.g. a spawned task panicked).
    #[error("internal error: {0}")]
    Internal(String),
}
