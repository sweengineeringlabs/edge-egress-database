//! Interface definition for the refinery-backed migration runner.
//!
//! This is the api-layer counterpart to `core::refinery::RefineryMigrationRunner`.
//! Consumers receive `impl MigrationRunner` from [`MigrationSvc::migration_runner`]
//! and never name this api-spec type directly.

/// Api-layer spec for a refinery-backed migration runner configuration.
///
/// The file stem `refinery_migration_runner` matches this type per rule 161.
pub struct RefineryMigrationRunner {
    /// The database URL (e.g. `postgres://…` or `sqlite:///…`).
    pub database_url: String,
    /// Path to the directory containing migration SQL files.
    pub migrations_dir: String,
}
