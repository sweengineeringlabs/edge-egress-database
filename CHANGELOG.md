# Changelog

All notable changes to `swe-edge-egress-database-migration` are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

## [0.2.1] — 2026-06-08

Datasource + migration bootstrap using **refinery** (migrations) + **deadpool**
(connection pool) over `tokio-postgres` / `rusqlite` drivers (issue #3, ADR-001).
`sqlx` was rejected due to RUSTSEC-2023-0071: `sqlx-mysql` introduces `rsa 0.9.x`
regardless of which feature flags are enabled; no upstream fix is available.
API-breaking, but released as a patch bump to match this repo's auto-tag scheme
(the crate is `0.x` and `auto-tag` patch-increments the latest tag on merge to `main`).

### Added
- `DatabaseConfig` — backend-owned `[database]` `OptionalSection` (ADR-006):
  `driver`, `url`, `max_connections`, `acquire_timeout_secs`, `idle_timeout_secs`,
  `migrations_dir`; `#[serde(deny_unknown_fields)]` + `validate_enabled`.
- `DriverKind` — neutral `sqlite` / `postgres` selector.
- `DbPool` — concrete deadpool-backed pool enum (`as_sqlite()` / `as_postgres()` /
  `close()`), an `spi/deadpool/` type surfaced via `saf/` (ADR-008).
- `MigrationSvc::connect_and_migrate(&DatabaseConfig) -> Result<DbPool, _>` —
  runs pending migrations via refinery on a dedicated connection, then opens a
  tuned deadpool and returns the ready pool.
- `MigrationSvc::connect(&DatabaseConfig)` — pool only, no migrations.
- `MigrationSvc::migration_runner(&DatabaseConfig)` — standalone refinery runner
  for callers that manage migrations and pooling separately.

### Changed
- Backend is now **refinery** (migration engine) + **deadpool** (async pool) over
  `tokio-postgres` and `rusqlite` drivers. `sqlx` was rejected:
  `sqlx-mysql → rsa 0.9.x` triggers RUSTSEC-2023-0071 regardless of feature flags.
- Migration file naming follows refinery convention: `V{n}__{description}.sql`
  (e.g. `V1__create_metrics.sql`).
- `MigrationSvc::migration_runner` now takes `&DatabaseConfig` (was `url, dir`).

### Removed
- `sqlx` and all transitive dependencies (`sqlx-mysql`, `rsa 0.9.x`).
  `cargo audit` exits 0; no RUSTSEC advisories in the dependency tree.

### Notes
- No `edge-domain` dependency (ADR-001). DB-backed `Repository` adapters and any
  `MigrationError → HandlerError` bridging live in the consumer (justobserv).
- SQLite consumers use `deadpool-sqlite`'s `interact(|conn| { ... })` API to
  execute blocking `rusqlite` calls on the pool's thread executor.
