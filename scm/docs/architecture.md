# Architecture — edge-egress-database

## Sequence

> At startup the composition root calls `connect_and_migrate(&DatabaseConfig)`:
> the crate opens a tuned `sqlx` pool, applies pending migrations in version
> order, and returns the ready pool. The consumer then queries through it.

```mermaid
sequenceDiagram
    participant Consumer as Consumer (composition root)
    participant MigrationSvc
    participant DbPool as sqlx pool (spi)
    participant Database

    Consumer->>MigrationSvc: connect_and_migrate(&DatabaseConfig)
    MigrationSvc->>MigrationSvc: select driver (DriverKind) + apply pool tuning
    MigrationSvc->>DbPool: open pool (max_connections, timeouts)
    DbPool->>Database: connect
    MigrationSvc->>Database: apply pending <version>_<desc>.sql in order (idempotent)
    Database-->>MigrationSvc: _sqlx_migrations updated
    MigrationSvc-->>Consumer: DbPool (ready, migrated)

    Consumer->>DbPool: as_sqlite()/as_postgres() → concrete sqlx pool
    Consumer->>Database: query (consumer's edge_domain::Repository adapter)
```

## Data Flow

> A `DatabaseConfig` drives pool construction and schema evolution; the output is
> a ready `DbPool` the consumer owns.

```mermaid
flowchart LR
    A["DatabaseConfig\n(driver, url,\nmax_connections, timeouts,\nmigrations_dir)"] --> B["MigrationSvc\n::connect_and_migrate"]

    B -->|select driver| D{DriverKind}
    D -->|sqlite| E["sqlx SqlitePool\n(spi/sqlx)"]
    D -->|postgres| F["sqlx PgPool\n(spi/sqlx)"]

    E --> G["run migrations\n(sqlx Migrator, in order)"]
    F --> G

    G -->|Ok| I["DbPool\n(ready, migrated)"]
    G -->|Err| J["MigrationError\n::Connection\n::MigrationsDirectoryNotFound\n::Apply\n::NotConfigured"]

    I --> K["consumer queries via\nas_sqlite()/as_postgres()"]
```
