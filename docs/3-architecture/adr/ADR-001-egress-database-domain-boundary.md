# ADR-001: egress-database is a Domain-Free Datasource + Migration Bootstrap (refinery + deadpool)

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

Three things had to be decided before implementing:

1. **The domain boundary.** Should this crate depend on `edge-domain` (the issue
   proposed integrating with its `HandlerError`), and should it ship the
   DB-backed `Repository` implementations?
2. **What it returns.** `justobserv` needs a connection pool to query through.
   What is the type and ownership of that pool, under ADR-008's rule that `api/`
   must name no external technology?
3. **Which SQL library.** The initial design specified `sqlx`; a post-pinning
   security audit blocked that choice.

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
an ORM and papering over Postgres/SQLite dialect differences — not a migration
crate). There is no useful neutral abstraction this crate can place *between* a
raw pool and domain's generic `Repository`.

### RUSTSEC-2023-0071 — `sqlx` is banned from this crate

`cargo audit` with `vulnerability = "deny"` is enforced across the assembler repo.
`sqlx` (the originally specified library) pulls in `sqlx-mysql` which depends on
`rsa 0.9.x`, regardless of whether only `sqlite` or `postgres` features are
enabled. `rsa 0.9.x` is affected by [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071)
(Marvin Attack timing sidechannel) with no upstream fix available as of this
writing. Adding `sqlx` to `Cargo.toml` — with any feature set — causes `cargo
audit` to fail.

The ban is a hard constraint, not a preference. It was first resolved in the edge
repo at commit `e51db44d` (2026-05-17) and must not be re-introduced.

---

## Decision

### 1. No `edge-domain` dependency; Repository impls live in the consumer

This crate does **not** depend on `edge-domain`. It does **not** ship
`impl Repository<T, Id>` for any entity, and does **not** map its errors into
`edge_domain::HandlerError`. Those belong to the consumer (`justobserv`), the
single place that depends on both `edge-domain` (the trait) and this crate (the
pool). The dependency direction is consumer → (domain, db); never db → domain.

### 2. This crate IS the datasource: refinery + deadpool, concrete pool returned

Because `sqlx` is banned (RUSTSEC-2023-0071), this crate uses:

- **[refinery](https://crates.io/crates/refinery)** — async migration engine.
  Runs pending `V{n}__{description}.sql` files against a dedicated connection,
  tracking applied migrations in `refinery_schema_history`. No `rsa` dependency.
- **[deadpool](https://crates.io/crates/deadpool)** — async connection pool.
  `deadpool-postgres` over `tokio-postgres` for Postgres; `deadpool-sqlite` over
  `rusqlite` for SQLite. Migrations run first on a dedicated connection; the pool
  is opened separately and returned only after migrations succeed.

The public surface returns a **concrete, configured, migrated pool**:

```rust
// saf/
pub async fn connect_and_migrate(cfg: &DatabaseConfig)
    -> Result<DbPool, MigrationError>;
// runs pending migrations on a dedicated connection →
// opens a deadpool with the configured sizing/timeouts →
// returns a READY pool the consumer queries through.
```

`DbPool` is a concrete deadpool-backed type defined in `spi/deadpool/` and
exported via `saf/`. This is **ADR-008-compliant**: `api/` stays technology-neutral
(it holds `MigrationRunner` / `MigrationStatus` ports and the neutral
`MigrationError`, none naming deadpool or refinery); the external-library type
lives in `spi/` and is surfaced through `saf/`, exactly as ADR-008 prescribes for
`spi` implementations.

A single blessed constructor is deliberate: a correct production pool needs
sizing, acquire/idle timeouts, and migrations-applied-before-first-query.
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
egress-database  (domain-free datasource + migration bootstrap)
  api/   MigrationRunner / MigrationStatus / MigrationError    ← neutral ports (ADR-008)
  api/   DatabaseConfig: OptionalSection                       ← backend-owned (ADR-006)
         ([database]: driver, url, max_connections, timeouts)
  core/  refinery migration runner                             ← runs V{n}__ files
  spi/deadpool/  DbPool + pool builder                         ← external lib confined here
  saf/   connect_and_migrate(&DatabaseConfig) -> Result<DbPool, MigrationError>

justobserv  (composition root)
  depends on edge-domain (Repository<T,Id>) + egress-database (DbPool, DatabaseConfig)

  let pool = connect_and_migrate(&cfg).await?;          // ready, migrated pool
  impl Repository<MetricRecord, MetricId> for PgMetricRepo { /* uses pool.as_postgres() */ }
```

The Domain contract is never passed through the DB crate. The consumer holds the
pool (from this crate) and the trait (from domain) and marries them in its own
`impl Repository<T, Id>`, querying through the concrete deadpool handle.

---

## Rationale

1. **The pool must be exposed, so expose it well.** Because the only useful data
   abstraction (`Repository`) is domain-owned and generic, this crate cannot offer
   a neutral query layer — it must return the real pool. Given that, returning a
   *correctly configured, already-migrated* pool is far more production-grade than
   shipping a config and leaving every consumer to build (and mis-build) its own.

2. **refinery + deadpool avoids RUSTSEC-2023-0071.** `sqlx` is the ecosystem-standard
   async pooled SQL toolkit, but its `sqlx-mysql` dependency (present regardless of
   features) introduces `rsa 0.9.x`, which has a timing sidechannel vulnerability
   with no upstream fix. `refinery` has no such chain. `deadpool` pools `tokio-postgres`
   and `rusqlite` directly — neither introduces `rsa`.

3. **Domain stays out (Rationale unchanged from the boundary decision).** The
   crate consumes no domain type; the only cited reason for the dep
   (`HandlerError` conversion) is a consumer-side boundary concern. Taking the dep
   would force every migration consumer to transitively pull in domain's contracts
   for no benefit — and message-broker, the precedent, takes no such dep.

4. **ADR-008 honoured honestly.** `api/` names neither refinery nor deadpool; the
   concrete pool is an `spi/deadpool/` type surfaced via `saf/`. Swapping the pool
   implementation later changes `spi/` + the `saf/` return type (a deliberate,
   semver-governed public choice) but not the `api/` ports.

5. **Separation of concerns between migrations and pooling.** refinery runs
   migrations on a dedicated connection and closes it. deadpool opens the pool
   independently. These are two distinct operations with different lifecycles;
   conflating them (as `sqlx`'s built-in migrator does) makes it harder to
   introspect or replace one without the other.

---

## Consequences

**Positive**

- One blessed, correctly-tuned, migrated pool; no per-consumer pool drift.
- `cargo audit` clean: no `rsa`, no `sqlx`, no `sqlx-mysql` in the dependency tree.
- `refinery` for migrations + `deadpool` for pooling are independently versioned and
  replaceable without touching `api/`.
- `api/` stays swappable; the backing libraries are `spi/` details.
- No db → domain coupling; faithful to ADR-006, ADR-008, and domain's own rule.

**Negative / accepted trade-offs**

- **Two libraries instead of one.** `sqlx` would have been one library for pooling
  + migrations. `refinery` + `deadpool` are two, each with their own lifecycle and
  upgrade path. The security constraint makes this the only viable choice.
- **`interact()` API.** `deadpool-sqlite` uses a blocking thread-pool model
  (`Object::interact(|conn| { ... })`) rather than async query methods. Consumer
  code is slightly more verbose. `deadpool-postgres` retains full async.
- **Crate identity broadens** from "migration runner" to "database datasource +
  migration bootstrap." A future rename (dropping the `-migration` suffix) should
  be considered; deferred to avoid churning the published tag mid-change.
- **Consumer is deadpool-coupled** by design — it queries through a concrete
  deadpool handle. This is intended (it is *choosing* the datasource by depending on
  this crate) and is confined to the consumer, not `api/`.
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
   drift, and contradicts issue #3's intent. Cheaper, but fails the
   operational-consistency bar.

4. **Own a parallel data-access trait here instead of using domain's
   `Repository`** (the literal message-broker pattern). Rejected: `Repository<T,
   Id>` already exists in domain as the canonical contract; a parallel port would
   fork the abstraction and force consumers to choose between two equivalent traits.

5. **`sqlx` (originally specified in issue #3).** Rejected: `sqlx` introduces
   `rsa 0.9.x` via `sqlx-mysql` regardless of feature flags. This triggers
   RUSTSEC-2023-0071 in `cargo audit` with no upstream fix. The ban is a hard
   constraint enforced across the assembler repo (first resolved at edge commit
   `e51db44d`, 2026-05-17).

6. **Return a bare `tokio_postgres::Pool` / `rusqlite::Connection` from `saf/` with
   no wrapper.** Acceptable under ADR-008 (it is an `spi`/`saf` concrete type, not
   an `api/` name), but the `DbPool` enum in `spi/deadpool/` is preferred so pool
   construction, tuning, and the migrate step have one owner and the public type is
   stable across internal driver changes.
