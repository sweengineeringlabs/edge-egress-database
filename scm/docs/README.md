# swe-edge-egress-database

## WHAT

Database schema migration runner for swe-edge services — runtime-agnostic migration execution with
versioning, status tracking, and optional rollback via the `refinery` backend.

Key capabilities:

- **`MigrationRunner`** — core trait: `run()`, `revert()`, `status() → Vec<MigrationStatus>`; dyn-safe; driver-agnostic
- **`Migration`** — VO with `version`, `description`, `applied_at`; constructors: `pending(version, desc)`, `applied(version, desc, timestamp)`
- **`MigrationStatus`** — per-migration applied/pending indicator for deployment validation
- **`MigrationError`** — enum: `NoMigrationToRevert`, `NotConfigured`, connection errors
- **`ApplicationConfigBuilder`** — fluent config assembly for database connection and migration paths
- Feature-flagged backends: `postgres`, `sqlite` (via `refinery`; disabled by default)

## WHY

| Problem | Solution |
|---------|----------|
| Schema migrations coupled to a specific database driver | `MigrationRunner` trait isolates driver specifics; the same application code targets postgres or sqlite via feature flags |
| Migration status unknown at deployment time | `status()` returns a typed `Vec<MigrationStatus>` — applied vs. pending is inspectable before any request is served |
| Rollback capability absent or ad-hoc | `revert()` on the trait; implementations that don't support rollback return `MigrationError::NotConfigured` explicitly |
| Diamond dep conflicts when migration types change | One crate, one tag — all consumers pin the same version; kgraph detects conflicts pre-commit |
