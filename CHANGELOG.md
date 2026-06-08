# Changelog

All notable changes to `swe-edge-egress-database-migration` are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

## [0.3.0] — 2026-06-08

Datasource + migration bootstrap on `sqlx` (issue #3, ADR-001). **Breaking.**

### Added
- `DatabaseConfig` — backend-owned `[database]` `OptionalSection` (ADR-006):
  `driver`, `url`, `max_connections`, `acquire_timeout_secs`, `idle_timeout_secs`,
  `migrations_dir`; `#[serde(deny_unknown_fields)]` + `validate_enabled`.
- `DriverKind` — neutral `sqlite` / `postgres` selector.
- `DbPool` — concrete `sqlx` pool (`as_sqlite()` / `as_postgres()` / `close()`),
  an `spi/` type surfaced via `saf/` (ADR-008).
- `MigrationSvc::connect_and_migrate(&DatabaseConfig) -> Result<DbPool, _>` —
  opens a tuned pool, runs pending migrations idempotently, returns the ready pool.
- `MigrationSvc::connect(&DatabaseConfig)` — pool only, no migrations.

### Changed
- Backend swapped from `refinery` + `tokio-postgres` + `rusqlite` to `sqlx`
  (one library for pooling + migrations + both drivers).
- `MigrationSvc::migration_runner` now takes `&DatabaseConfig` (was `url, dir`)
  and is backed by `sqlx`.
- Migration file naming follows `sqlx`: `<version>_<description>.sql`
  (was refinery's `V{n}__{description}.sql`).

### Removed
- `refinery`-backed runner and the `tokio-postgres` / `rusqlite` dependencies.

### Notes
- No `edge-domain` dependency (ADR-001). DB-backed `Repository` adapters and any
  `MigrationError → HandlerError` bridging live in the consumer (justobserv).
