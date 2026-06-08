//! Api-layer spec for a refinery-backed migration runner configuration.

/// Api-layer anchor for a refinery-backed migration runner.
///
/// Consumers receive `impl MigrationRunner` from the SAF and never
/// construct this type directly. Per SEA Rule 161 the file stem matches the
/// type name; per ADR-008 no external library name appears in `api/`.
#[expect(
    dead_code,
    reason = "SEA api/ interface anchor — counterpart spec; consumers receive impl MigrationRunner"
)]
pub struct RefineryMigrationRunner {
    /// The database URL (`postgres://…` or `sqlite:///…`).
    pub database_url: String,
    /// Directory containing migration `.sql` files.
    pub migrations_dir: String,
}
