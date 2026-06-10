# Changelog

All notable changes to `swe-edge-egress-database-migration` are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versions track the `swe-edge` release line.

## [0.2.1] — 2026-06-09

### Changed
- Replaced `sqlx` with `refinery` + `deadpool` to eliminate RUSTSEC-2023-0071 (`rsa` 0.9.x transitive dependency).
- `DbPool` moved to `api/types/` per SEA Rule 160 (public types belong in `api/`).
- `Migration` and `MigrationStatus` moved to `api/migration/types/` per SEA Rule 196.
- Removed explicit `[[test]]` entries from `Cargo.toml`; Cargo auto-discovers integration tests.

## [0.2.0] — 2026-06-04

### Added
- `DatabaseConfig` — typed `[database]` TOML section with `DriverKind`, `url`, pool settings, and `migrations_dir`.
- `MigrationSvc` SAF — `connect_and_migrate`, `connect`, `migration_runner`, `noop_migration_runner`.
- `MigrationRunner` trait with `run`, `revert`, `status`.
- SQLite backend (`sqlite` feature) via `refinery/rusqlite` + `deadpool-sqlite`.
- PostgreSQL backend (`postgres` feature) via `refinery/tokio-postgres` + `deadpool-postgres`.
