# ADR 0001 — egress-database stays a standalone leaf; no dependency on edge-domain

- **Status:** Accepted
- **Date:** 2026-06-08
- **Deciders:** edge platform engineering
- **Related:** [issue #3](https://github.com/sweengineeringlabs/edge-egress-database/issues/3) (MigrationRunner/MigrationSvc/DatabaseConfig — justobserv pillar persistence); `edge-domain` repository contract

## Context

`justobserv` needs durable storage for its four observability pillars (metrics,
logs, traces, monitoring snapshots). Today those backends use
`edge_domain::InMemoryRepository`, so data is lost on restart. Issue #3 asks this
crate (`swe-edge-egress-database-migration`) to provide the persistence plumbing.

The migration engine already exists in this crate and is independent of any other
swe-edge workspace:

- `MigrationRunner` trait + `RefineryMigrationRunner` (postgres + sqlite backends)
- `MigrationSvc` factory, `MigrationError`, `Migration`, `MigrationStatus`

`edge-domain` owns the persistence **contract** — `Repository<T, Id>` and
`QueryableRepository<T, Id>` — plus an `InMemoryRepository`. The trait's own
documentation states: *"Implementations live in infrastructure crates — never in
`edge-domain`. `edge-domain` owns only this contract."*

Issue #3 originally proposed that this crate *"integrates with `edge-domain` —
`HandlerError` variants for migration failures."* That implies a new dependency
edge **egress-database → edge-domain**. This ADR decides whether to take that
dependency.

## Decision

**`swe-edge-egress-database-migration` does NOT depend on `edge-domain`.** It
remains a standalone infrastructure leaf.

Its scope for issue #3 is exactly three things:

1. `DatabaseConfig` — a TOML `[database]` section type implementing
   `swe_edge_configbuilder::OptionalSection`.
2. `connect_and_migrate(cfg: DatabaseConfig) -> Result<Pool, MigrationError>` —
   open a connection pool and run pending migrations idempotently, returning the
   pool handle.
3. The existing migration run (already implemented).

The concrete `impl Repository<T, Id>` (the DB-backed adapter) is **out of scope
for this crate**. It belongs to the consumer (`justobserv`), which is the single
place that depends on both `edge-domain` (for the trait) and this crate (for the
pool).

## Rationale

1. **This crate cannot implement justobserv's repositories.** `Repository<T, Id>`
   is generic over the entity type; a DB-backed impl must map a concrete entity
   (`MetricRecord`, `LogRecord`, …) to columns. Those entities are owned by
   `justobserv`, not here. A generic `Repository<T, Id>` over arbitrary `T` is not
   practical without per-entity SQL, so the adapters belong with the entities.

2. **Error bridging is a consumer concern.** This crate already exposes a complete
   `MigrationError`. Converting it into `edge_domain::HandlerError` is the caller's
   job at its own boundary. Reaching *up* into a consumer's error enum from a leaf
   crate is leaky, backwards coupling — the migration crate should not know
   `HandlerError` exists.

3. **Neither missing piece needs a domain symbol.** `DatabaseConfig` needs only
   `configbuilder`; `connect_and_migrate -> Pool` needs only the pool library.
   Zero `edge-domain` types are involved in the work that actually lands here.

4. **Cohesion and blast radius.** The crate is, by name and metadata, a database
   migration runner (`service_type = "processor"`). Taking the `edge-domain`
   dependency would force every migration consumer to transitively pull in
   domain's `Handler` / `Repository` / `HandlerError` contracts and couple
   migration releases to domain's cadence — to buy nothing but an error-conversion
   convenience.

## Architecture

```
            depends on                         depends on
 justobserv ───────────▶ edge-domain   justobserv ───────────▶ egress-database
 (adapter /              (Repository<T,Id>        (DatabaseConfig,
  composition root)       trait)                   connect_and_migrate -> Pool)
```

The Domain contract is never passed through the DB crate. The consumer holds the
pool (from this crate) and the trait (from domain), and marries them in its own
`impl Repository<T, Id>`:

```rust
use edge_domain::Repository;                  // contract
use swe_edge_egress_database_migration::connect_and_migrate; // pool

struct PostgresMetricRepository { pool: Pool }
impl Repository<MetricRecord, MetricId> for PostgresMetricRepository { /* uses self.pool */ }

let pool = connect_and_migrate(cfg).await?;
let repo = PostgresMetricRepository { pool };
```

## Consequences

**Positive**

- This crate stays a reusable leaf consumable for migrations alone.
- No upward/leaky coupling; dependency direction is consumer → (domain, db).
- Faithful to domain's stated rule that implementations live in infra crates.

**Negative / accepted trade-offs**

- `justobserv` (not this crate) owns the `Repository` adapters and any
  `MigrationError → HandlerError` mapping. This must be reflected in justobserv's
  scope.
- Issue #3 is re-scoped: the `edge-domain` integration line and any DB-backed
  `Repository` deliverable are removed from this crate's acceptance criteria.

## Alternatives considered

- **egress-database → edge-domain (the issue's original framing).** Rejected: the
  only cited driver (`HandlerError` variants) is a consumer-side boundary concern,
  and the crate would still not consume any domain type, so the dependency adds
  coupling with no benefit (see Rationale 1–4).
