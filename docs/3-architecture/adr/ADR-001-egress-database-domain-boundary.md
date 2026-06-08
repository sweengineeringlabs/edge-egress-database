# ADR-001: egress-database Stays Domain-Free — Repository Impls Live in the Consumer

**Status:** Accepted
**Date:** 2026-06-08
**Deciders:** edge platform engineering
**GitHub Issue:** [edge-egress-database#3](https://github.com/sweengineeringlabs/edge-egress-database/issues/3)
**Governed by:** [ADR-006 (Backend-Owned OptionalSection)](../../../../../docs/3-architecture/adr/ADR-006-backend-owned-optional-section.md), [ADR-008 (SEA API Neutrality & SPI)](../../../../../docs/3-design/ADR-008-sea-api-neutrality-spi-implementations.md)

---

## Context

`justobserv` needs durable storage for its four observability pillars (metrics,
logs, traces, monitoring snapshots). Today those backends use
`edge_domain::InMemoryRepository`, so data is lost on restart. Issue #3 asks this
crate (`swe-edge-egress-database-migration`) to supply the persistence plumbing.

The migration engine already exists here and is independent of every other
workspace: `MigrationRunner` + `RefineryMigrationRunner` (postgres + sqlite),
`MigrationSvc`, `MigrationError`, `Migration`, `MigrationStatus`.

`edge-domain` owns the persistence **contract** — `Repository<T, Id>` /
`QueryableRepository<T, Id>` plus an `InMemoryRepository`. Its own trait doc
states: *"Implementations live in infrastructure crates — never in
`edge-domain`. `edge-domain` owns only this contract."*

Issue #3 originally proposed this crate *"integrate with `edge-domain` —
`HandlerError` variants for migration failures,"* implying a new dependency edge
**egress-database → edge-domain**. This ADR decides the boundary. It does **not**
re-decide config ownership or technology neutrality — those are already settled
monorepo-wide by ADR-006 and ADR-008 respectively; this ADR applies them.

### Precedent: swe-edge-message-broker

`swe-edge-message-broker` is the reference implementation of ADR-006 and is
structurally identical to this crate. It establishes the pattern we follow:

- **No `edge-domain` dependency.** Its only non-std deps are `configbuilder`,
  `serde`, `futures`, `thiserror`, `tracing`.
- **Owns its config** — `MessageBrokerConfig: OptionalSection` (`[message_broker]`,
  a `BackendKind` enum, `validate_enabled`, `metadata`, `deny_unknown_fields`).
- **Owns its port trait** — `MessageBroker`, with a `noop_broker()` default and an
  `spi/` extension point; production backends (nats/kafka) are constructed
  elsewhere and injected.
- **Leaks no raw handle.** Consumers get `Box<dyn MessageBroker>`, never a raw
  socket or client.

The one place message-broker does not map 1:1: its payload (`Message`) is
monomorphic, so a single trait fully abstracts it. Data access is *polymorphic*
(`Repository<T, Id>` is generic over the entity), and that abstraction is owned
by `edge-domain` — which is exactly why the concrete impl cannot live here.

---

## Decision

**`swe-edge-egress-database-migration` does NOT depend on `edge-domain`.** It
remains a standalone infrastructure leaf. Its scope for issue #3:

1. **`DatabaseConfig: OptionalSection`** — backend-owned `[database]` section
   (driver + url), per **ADR-006**. Mirrors `MessageBrokerConfig`.
2. **A `saf/` `from_config` factory** that opens a connection pool and runs
   pending migrations idempotently, returning a **technology-neutral** pool
   handle.
3. The existing migration run (already implemented).

The concrete `impl Repository<T, Id>` (the DB-backed adapter) and any
`MigrationError → HandlerError` mapping are **out of scope for this crate**. They
belong to the consumer (`justobserv`), the single place that depends on both
`edge-domain` (the trait) and this crate (the pool handle).

### Applying ADR-008 to the pool handle

Per ADR-008, `api/` may name no external technology — `Sqlx*` is explicitly
forbidden — and *"swapping the backing library must not change a single file
under `api/`."* Therefore:

- The pool/connection handle is a **neutral type/trait in `api/`** (no `Sqlx`,
  `Deadpool`, `Pg`, etc. in its name or signature).
- The concrete pool implementation (`sqlx`, `deadpool`, …) lives in
  **`spi/{technology}/`** and is exported only through `saf/`.
- The chosen pool library is an `spi/` implementation detail, not part of the
  public contract. This resolves "sqlx vs deadpool" as a non-architectural,
  swappable choice.

Returning a bare `sqlx::Pool` from a `saf/` function would violate ADR-008 and is
rejected.

---

## Architecture

```
            depends on                          depends on
 justobserv ──────────▶ edge-domain    justobserv ──────────▶ egress-database
 (adapter /             (Repository<T,Id>        (DatabaseConfig + neutral
  composition root)      trait)                   pool handle + migrations)
```

The Domain contract is never passed through the DB crate. The consumer holds the
neutral pool handle (from this crate) and the trait (from domain) and marries
them in its own `impl Repository<T, Id>`:

```rust
use edge_domain::Repository;                              // contract (domain)
use swe_edge_egress_database_migration::{DatabaseConfig, DatabasePool}; // neutral handle (this crate)

struct PostgresMetricRepository { pool: DatabasePool }
impl Repository<MetricRecord, MetricId> for PostgresMetricRepository { /* uses self.pool */ }

// from_config opens the pool and runs migrations; returns a neutral DatabasePool
let pool = DatabaseSvc::from_config(cfg).await?;
let repo = PostgresMetricRepository { pool };
```

`DatabasePool` here denotes the neutral `api/` handle; its `sqlx`/`deadpool`
backing lives in `spi/` and never appears in a public signature.

---

## Rationale

1. **This crate cannot implement justobserv's repositories.** `Repository<T, Id>`
   is generic over the entity; a DB-backed impl must map a concrete entity
   (`MetricRecord`, `LogRecord`, …) to columns. Those entities are owned by
   `justobserv`. A generic impl over arbitrary `T` is impractical without
   per-entity SQL, so the adapters belong with the entities.

2. **Error bridging is a consumer concern.** This crate exposes a complete
   `MigrationError`. Converting it into `edge_domain::HandlerError` is the
   caller's boundary job; a leaf reaching up into a consumer's error enum is
   backwards coupling. (The sole reason issue #3 cited for the domain dep.)

3. **Neither deliverable needs a domain symbol.** `DatabaseConfig` needs only
   `configbuilder`; the pool handle needs only the pool library, hidden in
   `spi/`. Zero `edge-domain` types are involved.

4. **Cohesion and blast radius.** Taking the dep would force every migration
   consumer to transitively pull in domain's `Handler` / `Repository` /
   `HandlerError` contracts and couple migration releases to domain's cadence —
   for an error-conversion convenience. The precedent (message-broker) takes no
   such dep.

---

## Consequences

**Positive**

- Stays a reusable leaf consumable for migrations alone; dependency direction is
  consumer → (domain, db), never db → domain.
- Faithful to ADR-006 (backend-owned config), ADR-008 (api neutrality), and
  domain's own rule that implementations live in infra crates.
- "sqlx vs deadpool" is demoted to an `spi/` detail, not a public contract.

**Negative / accepted trade-offs**

- `justobserv` (not this crate) owns the `Repository` adapters and any
  `MigrationError → HandlerError` mapping — reflected in justobserv's scope
  (`sweengineeringlabs/justobserv#20`).
- This crate must complete its ADR-006 Phase-2 migration: replace the current
  `database_url: impl Into<String>` shape with `DatabaseConfig: OptionalSection`.
- A neutral `api/` pool handle + `spi/` impl is more structure than returning a
  raw pool — but it is what ADR-008 requires.

---

## Alternatives Considered

1. **egress-database → edge-domain (issue #3's original framing).** Rejected: the
   only cited driver (`HandlerError` variants) is a consumer-side boundary
   concern, and the crate would still consume no domain type — pure coupling for
   no benefit (Rationale 1–4).

2. **Return a bare `sqlx::Pool` from `saf/`.** Rejected: violates ADR-008
   (`Sqlx*` forbidden in `api/`; backing library must be swappable without
   touching `api/`).

3. **Own a data-access trait here instead of using domain's `Repository`**
   (the literal message-broker pattern). Rejected: `Repository<T, Id>` already
   exists in domain and is the canonical contract; duplicating a parallel
   data-access port here would fork the abstraction and force consumers to choose
   between two equivalent traits.
