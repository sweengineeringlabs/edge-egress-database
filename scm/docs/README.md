# swe-edge-egress-database

## WHAT

Database datasource + schema-migration bootstrap for swe-edge services. It owns
the `[database]` config contract, opens a tuned `sqlx` connection pool, applies
pending migrations idempotently, and returns a ready pool the consumer queries
through. Forward-only migrations; no rollback.

Key capabilities:

- **`DatabaseConfig`** — backend-owned `[database]` `OptionalSection` (ADR-006):
  `driver`, `url`, pool tuning (`max_connections`, `acquire_timeout_secs`,
  `idle_timeout_secs`), `migrations_dir`; `#[serde(deny_unknown_fields)]`
- **`MigrationSvc::connect_and_migrate(&cfg)`** — opens the pool, runs pending
  migrations, returns a ready **`DbPool`**
- **`MigrationSvc::connect(&cfg)`** — pool only (no migrations)
- **`DbPool`** — concrete `sqlx` pool (`as_sqlite()` / `as_postgres()`), an `spi/`
  type surfaced via `saf/` (ADR-008); `api/` names no driver library
- **`MigrationRunner`** — neutral port: `run()`, `revert()`, `status() →
  Vec<MigrationStatus>`; dyn-safe. `noop_migration_runner()` is always available
- **`Migration` / `MigrationStatus`** — value objects for applied/pending state
- **`MigrationError`** — `Connection`, `Apply`, `MigrationsDirectoryNotFound`,
  `NotConfigured`, `NoMigrationToRevert`, `Internal`
- Feature-flagged backends: `postgres`, `sqlite` (via `sqlx`; disabled by default)

Migration files use `sqlx` naming: `<version>_<description>.sql`.

## WHY

| Problem | Solution |
|---------|----------|
| Every consumer hand-builds (and mis-tunes) its own pool | One blessed `connect_and_migrate` returns a sizing/timeout-correct, already-migrated pool |
| Config shape duplicated and version-skewed across apps | Backend owns `DatabaseConfig: OptionalSection` — one canonical `[database]` contract (ADR-006) |
| Driver library leaking into the contract surface | Concrete `sqlx` pool confined to `spi/`, surfaced via `saf/`; `api/` is technology-neutral (ADR-008) |
| Persistence coupling the migration crate to domain | No `edge-domain` dependency; DB-backed `Repository` adapters live in the consumer (ADR-001) |
| Migration status unknown at deploy time | `status()` returns a typed `Vec<MigrationStatus>` — applied vs. pending before serving traffic |
