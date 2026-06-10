# swe-edge-egress-database-migration

Database datasource + migration bootstrap for swe-edge.

Owns the `[database]` config contract (`DatabaseConfig`), runs pending schema
migrations via **refinery** (RUSTSEC-2023-0071-clean; no `rsa` dependency), and
returns a ready **deadpool** `DbPool` the consumer queries through.

## Features

| Feature    | Driver     | URL scheme     |
|------------|------------|----------------|
| _(none)_   | —          | noop only      |
| `sqlite`   | SQLite     | `sqlite:///…`  |
| `postgres` | PostgreSQL | `postgres://…` |

Migration files follow **refinery** naming: `V{n}__{description}.sql`
(e.g. `V1__create_metrics.sql`).

## Quick start

```toml
# Cargo.toml
swe-edge-egress-database-migration = { ..., features = ["sqlite"] }
```

```toml
# application.toml
[database]
driver         = "sqlite"
url            = "sqlite:///var/lib/myapp/data.db"
migrations_dir = "./migrations"
```

```rust
use swe_edge_egress_database_migration::{DatabaseConfig, DriverKind, MigrationSvc};

let cfg = DatabaseConfig {
    driver: DriverKind::Sqlite,
    url: "sqlite:///./data.db".into(),
    max_connections: 5,
    acquire_timeout_secs: 30,
    idle_timeout_secs: None,
    migrations_dir: Some("./migrations".into()),
};

let pool = MigrationSvc::connect_and_migrate(&cfg).await?;
// pool.as_sqlite() yields the deadpool-sqlite pool for Repository adapters.
```

## License

MIT OR Apache-2.0
