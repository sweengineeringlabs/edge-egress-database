# swe-edge-egress-database

> **TLDR:** Database datasource + migration bootstrap for swe-edge — `DatabaseConfig` (backed `[database]` config) + `connect_and_migrate` returning a configured, migrated `sqlx` pool behind `postgres` / `sqlite` feature flags. See [Overview](scm/docs/README.md) for details.

Domain-free infrastructure leaf: it owns the `[database]` config contract, opens
a tuned `sqlx` connection pool, applies pending schema migrations, and hands back
a ready `DbPool` the consumer queries through. DB-backed `edge_domain::Repository`
adapters live in the consumer (see [ADR-001](docs/3-architecture/adr/ADR-001-egress-database-domain-boundary.md)).

## Workspace layout

```
scm/           <- Cargo package root (Pattern A: single-crate library)
  main/src/    <- library source
  tests/       <- integration tests
  config/      <- TOML configuration defaults
  examples/    <- runnable examples
  docs/        <- crate-level docs
```

## Features

| Flag       | What it enables                          |
|------------|------------------------------------------|
| `postgres` | `sqlx`-backed PostgreSQL pool + migrator |
| `sqlite`   | `sqlx`-backed SQLite pool + migrator     |

Default is no features (only the no-op runner). The concrete `sqlx` pool lives in
`spi/` and is surfaced via `saf/`; the `api/` surface names no driver library
(ADR-008).

## Quick start

```toml
[dependencies]
swe-edge-egress-database-migration = { git = "...", features = ["sqlite"] }
```

```rust,no_run
use swe_edge_egress_database_migration::{DatabaseConfig, DriverKind, MigrationSvc};

# async fn run() -> Result<(), swe_edge_egress_database_migration::MigrationError> {
let cfg = DatabaseConfig {
    driver: DriverKind::Sqlite,
    url: "sqlite:///./obsrv.db".into(),
    max_connections: 5,
    acquire_timeout_secs: 30,
    idle_timeout_secs: None,
    migrations_dir: Some("./migrations".into()),
};
let pool = MigrationSvc::connect_and_migrate(&cfg).await?;
// pool.as_sqlite() / pool.as_postgres() → concrete sqlx pool for Repository impls
# Ok(())
# }
```

## Documentation

| Document | Description |
|----------|-------------|
| [Overview](scm/docs/README.md) | WHAT + WHY — capabilities and design rationale |
| [ADR-001](docs/3-architecture/adr/ADR-001-egress-database-domain-boundary.md) | Domain boundary + sqlx datasource decision |
