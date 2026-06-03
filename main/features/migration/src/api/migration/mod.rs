//! Migration value objects.

// SEA domain-grouped layout: the `migration` type lives in api/migration/migration.rs
// (file stem matches the type per rule 161), which clippy reads as module inception.
#[allow(clippy::module_inception)]
pub mod migration;
pub mod migration_status;

pub use migration::Migration;
pub use migration_status::MigrationStatus;
