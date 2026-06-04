//! `MigrationSvc` — public handle for the migration factory surface.

/// Public handle used as the `impl Processor` receiver in SAF methods.
///
/// Callers never construct this type directly — the SAF factory functions
/// are associated methods (`MigrationSvc::noop_migration_runner()`, etc.).
pub struct MigrationSvc;
