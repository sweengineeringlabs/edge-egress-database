# ADR-001: egress-database is a Domain-Free `sqlx` Datasource + Migration Bootstrap

**Status:** Accepted
**Date:** 2026-06-08
**Deciders:** edge platform engineering
**GitHub Issue:** [edge-egress-database#3](https://github.com/sweengineeringlabs/edge-egress-database/issues/3)
**Governed by:** [ADR-006 (Backend-Owned OptionalSection)](../../../../../docs/3-architecture/adr/ADR-006-backend-owned-optional-section.md), [ADR-008 (SEA API Neutrality & SPI)](../../../../../docs/3-design/ADR-008-sea-api-neutrality-spi-implementations.md)

---

## Context

`justobserv` needs durable storage for its four observability pillars (metrics,
logs, traces, monitoring snapshots). Those backends currently use
`edge_domain::InMemoryRepository`, so data is lost on restart. Issue #3 asks this
crate to supply the persistence plumbing.

Two things had to be decided before implementing:

1. **The domain boundary.** Should this crate depend on `edge-domain` (the issue
   proposed integrating with its `HandlerError`), and should it ship the
   DB-backed `Repository` implementations?
2. **What it returns.** `justobserv` needs a connection pool to query through.
   What is the type and ownership of that pool, under ADR-008's rule that `api/`
   must name no external technology?

`edge-domain` owns the persistence **contract** — `Repository<T, Id>` /
`QueryableRepository<T, Id>` — and its trait doc states: *"Implementations live in
infrastructure crates — never in `edge-domain`."* `swe-edge-message-broker` is
the reference ADR-006 implementation and takes **no** `edge-domain` dependency;
it owns its config (`MessageBrokerConfig: OptionalSection`), its port trait, and
hides its backends behind `dyn MessageBroker`.

### The polymorphic-T constraint

message-broker can fully hide its backend behind one trait because its payload
(`Message`) is **monomorphic** — `publish`/`subscribe` is the complete useful
surface. Data access is **polymorphic**: the useful abstraction is
`Repository<T, Id>`, generic over the entity, and it is owned by `edge-domain`.
A database pool's only value to a repository is running **typed queries**, which
is inherently SQL/driver-shaped.

This rules out a "technology-neutral pool handle" in `api/`: such a handle is
either opaque (the consumer cannot query through it without recovering the
concrete type — useless) or it grows into a full neutral query API (re-inventing
`sqlx`'s `Executor` and papering over Postgres/SQLite dialect differences — an
ORM, not a migration crate). There is no useful neutral abstraction this crate
can place *between* a raw pool and domain's generic `Repository`.

---

## Decision

### 1. No `edge-domain` dependency; Repository impls live in the consumer

This crate does **not** depend on `edge-domain`. It does **not** ship
`impl Repository<T, Id>` for any entity, and does **not** map its errors into
`edge_domain::HandlerError`. Those belong to the consumer (`justobserv`), the
single place that depends on both `edge-domain` (the trait) and this crate (the
pool). The dependency direction is consumer → (domain, db); never db → domain.

### 2. This crate IS the datasource: `sqlx` pool + migrations, concrete pool returned

This crate adopts **`sqlx`** as its single SQL backend, replacing
`refinery` + `tokio-postgres` + `rusqlite`. `sqlx` provides pooling **and**
migrations in one maintained library, and issue #3's own feature table already
specifies `sqlx` + SQLite / `sqlx` + Postgres — the prior `refinery` stack was
the divergence.

The public surface returns a **concrete, configured, migrated pool**:

```rust
// saf/
pub async fn connect_and_migrate(cfg: &DatabaseConfig)
    -> Result<DbPool, MigrationError>;
// builds a pool with the configured sizing/timeouts → runs pending
// migrations → returns a READY pool the consumer queries through.
```

`DbPool` is a concrete `sqlx`-backed type defined in `spi/sqlx/` and exported via
`saf/`. This is **ADR-008-compliant**: `api/` stays technology-neutral (it holds
the `MigrationRunner` / `MigrationStatus` ports and the neutral `MigrationError`,
none naming `sqlx`); the external-library type lives in `spi/` and is surfaced
through `saf/`, exactly as ADR-008 prescribes for `spi` implementations (the same
way ingress surfaces `TonicGrpcServer`). What ADR-008 forbids — a `Sqlx*` name in
`api/` — is not done.

A single blessed constructor is deliberate: a correct production pool needs
sizing, acquire/idle timeouts, TLS, and migrations-applied-before-first-query.
Centralising that here makes the correct path the easy path and prevents
per-consumer configuration drift.

### 3. Config is backend-owned (`DatabaseConfig: OptionalSection`) — ADR-006

This crate owns the `[database]` TOML contract as `DatabaseConfig: OptionalSection`
(canonical `section_name() = "database"`, a `DriverKind` enum, pool-tuning fields,
`validate_enabled`, `metadata`, `#[serde(deny_unknown_fields)]`), mirroring
`MessageBrokerConfig`. This completes the ADR-006 Phase-2 migration, replacing the
current `database_url: impl Into<String>` shape.

---

## Architecture

```
egress-database  (domain-free sqlx datasource + migration bootstrap)
  api/   MigrationRunner / MigrationStatus / MigrationError    ← neutral ports (ADR-008)
  api/   DatabaseConfig: OptionalSection                       ← backend-owned (ADR-006)
         ([database]: driver, url, max_connections, timeouts)
  spi/sqlx/  DbPool + sqlx migrator                            ← external lib confined here
  saf/   connect_and_migrate(&DatabaseConfig) -> Result<DbPool, MigrationError>

justobserv  (composition root)
  depends on edge-domain (Repository<T,Id>) + egress-database (DbPool, DatabaseConfig)

  let pool = connect_and_migrate(&cfg).await?;          // ready, migrated pool
  impl Repository<MetricRecord, MetricId> for PgMetricRepo { /* uses pool (sqlx) */ }
```

The Domain contract is never passed through the DB crate. The consumer holds the
pool (from this crate) and the trait (from domain) and marries them in its own
`impl Repository<T, Id>`, using `sqlx` directly for the per-entity SQL.

---

## Rationale

1. **The pool must be exposed, so expose it well.** Because the only useful data
   abstraction (`Repository`) is domain-owned and generic, this crate cannot offer
   a neutral query layer — it must return the real pool. Given that, returning a
   *correctly configured, already-migrated* pool is far more production-grade than
   shipping a config and leaving every consumer to build (and mis-build) its own.

2. **`sqlx` unifies the stack and matches intent.** It is the ecosystem-standard
   async pooled SQL toolkit with built-in migrations; one library covers pool +
   migrate + both drivers. It is also what issue #3 specified.

3. **Domain stays out (Rationale unchanged from the boundary decision).** The
   crate consumes no domain type; the only cited reason for the dep
   (`HandlerError` conversion) is a consumer-side boundary concern. Taking the dep
   would force every migration consumer to transitively pull in domain's contracts
   for no benefit — and message-broker, the precedent, takes no such dep.

4. **ADR-008 honoured honestly.** `api/` names no `sqlx`; the concrete pool is an
   `spi/` type surfaced via `saf/`. Swapping `sqlx` later changes `spi/` + the
   `saf/` return type (a deliberate, semver-governed public choice) but not the
   `api/` ports.

---

## Consequences

**Positive**

- One blessed, correctly-tuned, migrated pool; no per-consumer pool drift.
- Single SQL library (`sqlx`) for pool + migrations + both drivers.
- No db → domain coupling; faithful to ADR-006, ADR-008, and domain's own rule.
- `api/` stays swappable; the backing library is an `spi/` detail.

**Negative / accepted trade-offs**

- **Rewrite cost.** Replacing `refinery`/`tokio-postgres`/`rusqlite` with `sqlx`
  rewrites the runner and its integration tests against `sqlx`'s migrator.
- **Crate identity broadens** from "migration runner" to "database datasource +
  migration bootstrap." A future rename (dropping the `-migration` suffix) should
  be considered; deferred to avoid churning the published tag mid-change.
- **Consumer is `sqlx`-coupled** by design — it queries through a concrete `sqlx`
  pool. This is intended (it is *choosing* the datasource by depending on this
  crate) and is confined to the consumer, not `api/`.
- `justobserv` owns the `Repository` adapters and any `MigrationError →
  HandlerError` mapping (tracked in `sweengineeringlabs/justobserv#20`).

---

## Alternatives Considered

1. **egress-database → edge-domain (issue #3's original framing).** Rejected: the
   only cited driver (`HandlerError` variants) is a consumer boundary concern, and
   the crate consumes no domain type — pure coupling for no benefit.

2. **Technology-neutral pool handle in `api/`.** Rejected: a database pool's value
   is typed queries, which are inherently technology-shaped; a neutral handle is
   either opaque-and-useless or an ORM re-implementation. The polymorphic-`T`
   nature of `Repository` means there is no useful neutral abstraction to place
   between the pool and domain's contract.

3. **Migration-only leaf (config + migrations, no pool).** Rejected: it pushes
   pool construction onto every consumer, inviting timeout/TLS/sizing/ordering
   drift, and contradicts issue #3's `sqlx` + pool intent. Cheaper, but fails the
   operational-consistency bar.

4. **Own a parallel data-access trait here instead of using domain's
   `Repository`** (the literal message-broker pattern). Rejected: `Repository<T,
   Id>` already exists in domain as the canonical contract; a parallel port would
   fork the abstraction and force consumers to choose between two equivalent
   traits.

5. **Return a bare `sqlx::Pool` from `saf/` with no wrapper.** Acceptable under
   ADR-008 (it is an `spi`/`saf` concrete type, not an `api/` name), but a thin
   `DbPool` newtype in `spi/sqlx/` is preferred so pool construction, tuning, and
   the migrate step have one owner and the public type is stable across internal
   `sqlx` changes.
