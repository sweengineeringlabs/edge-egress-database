# swe-edge-egress-database

> **TLDR:** Database-agnostic schema migration runner for swe-edge — `MigrationRunner` trait with `postgres` and `sqlite` feature-flagged backends; status tracking and optional rollback. See [Overview](scm/docs/README.md) for details.

Database migration runner for swe-edge — `MigrationRunner` trait with
refinery-backed implementations behind `postgres` / `sqlite` feature flags.

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

| Flag       | What it enables                         |
|------------|-----------------------------------------|
| `postgres` | `tokio-postgres`-backed runner          |
| `sqlite`   | `rusqlite` (bundled) runner             |

## Quick start

```toml
[dependencies]
swe-edge-egress-database-migration = { git = "...", features = ["sqlite"] }
```

## Documentation

| Document | Description |
|----------|-------------|
| [Overview](scm/docs/README.md) | WHAT + WHY — capabilities and design rationale |
