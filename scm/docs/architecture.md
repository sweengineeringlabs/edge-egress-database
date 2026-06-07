# Architecture — edge-egress-database

## Sequence

> The runtime creates a `MigrationRunner` from a connection URL at startup; the runner applies pending SQL migrations in version order.

```mermaid
sequenceDiagram
    participant Runtime
    participant MigrationSvc
    participant MigrationRunner
    participant Database

    Runtime->>MigrationSvc: migration_runner(db_url, migrations_dir)
    MigrationSvc->>MigrationSvc: detect DB type from URL scheme (sqlite:// / postgres://)
    MigrationSvc-->>Runtime: Box<dyn MigrationRunner>

    Runtime->>MigrationRunner: status()
    MigrationRunner->>Database: SELECT applied migrations
    Database-->>MigrationRunner: Vec<MigrationStatus>
    MigrationRunner-->>Runtime: Vec<MigrationStatus>

    Runtime->>MigrationRunner: run()
    MigrationRunner->>Database: apply each pending .sql in version order
    Database-->>MigrationRunner: Ok per migration
    MigrationRunner-->>Runtime: Result<Vec<Migration>, MigrationError>

    opt rollback
        Runtime->>MigrationRunner: revert()
        MigrationRunner->>Database: undo last migration
        MigrationRunner-->>Runtime: Result<Migration, MigrationError>
    end
```

## Data Flow

> A `(db_url, migrations_dir)` pair drives schema evolution; the output is the list of migrations applied in this run.

```mermaid
flowchart LR
    A["db_url: &str\n(sqlite:///./dev.db\npostgres://host/db)"] --> B["MigrationSvc\n::migration_runner"]
    C["migrations_dir: &str\n(./migrations/*.sql\nversioned by timestamp)"] --> B

    B -->|detect scheme| D{DB type}
    D -->|sqlite| E["SQLite\nMigrationRunner"]
    D -->|postgres| F["Postgres\nMigrationRunner"]

    E --> G["status() → Vec<MigrationStatus>"]
    F --> G

    G --> H["run()\napply pending in order"]
    H -->|Ok| I["Vec<Migration>\napplied list"]
    H -->|Err| J["MigrationError\n::Connection\n::NotConfigured\n::SqlError"]
```
