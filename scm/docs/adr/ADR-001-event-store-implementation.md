# ADR-001: EventStore Implementation — egress/database

**Status:** Accepted  
**Date:** 2026-06-12  
**Governing ADR:** [ADR-018](https://github.com/sweengineeringlabs/edge/blob/main/docs/3-architecture/adr/ADR-018-event-sourcing-pipeline.md) — Event Sourcing Pipeline

---

## Mandate

Implement `EventStore<E>` from `edge-domain-event` for each database backend supported by `egress/database`.

---

## Dependency

```toml
edge-domain-event = { git = "https://github.com/sweengineeringlabs/edge-domain" }
```

No `features` key — `edge-domain-event` is a zero-feature crate.

---

## Schema

Each backend maintains an `events` table (or equivalent collection):

| Column | Type | Notes |
|---|---|---|
| `id` | UUID / serial | Surrogate key |
| `aggregate_id` | TEXT | Stream identifier |
| `sequence` | BIGINT | Monotonically increasing per `aggregate_id` |
| `event_type` | TEXT | `DomainEvent::event_type()` |
| `payload` | BYTEA / JSONB | Serialised event body |
| `occurred_at` | TIMESTAMPTZ | `DomainEvent::occurred_at()` |

---

## Optimistic concurrency

`EventStore::append` with `ExpectedVersion::Exact(n)`:

1. Read `MAX(sequence)` for `aggregate_id` within the transaction.
2. If `MAX(sequence) != n`, return `EventStoreError::Conflict`.
3. Insert new rows with `sequence = n+1, n+2, ...`.

`ExpectedVersion::Any` skips the read step — allowed only in tests and data migrations.

---

## `append` return value

Returns the new `MAX(sequence)` after the write — callers use this as the next `ExpectedVersion::Exact`.

---

## Backends

| Backend | Feature flag | Notes |
|---|---|---|
| Postgres | `postgres` | Uses `FOR UPDATE` row lock on sequence check |
| SQLite | `sqlite` | Uses `BEGIN IMMEDIATE` transaction |
| In-memory | always-on | `InMemoryEventStore<E>` from `edge-domain-event` SAF |

---

## Testing

- Unit tests use `EventFactory::in_memory_store::<E>()` — no backend.
- Integration tests use a per-test schema prefix (Postgres) or a per-test in-memory file (SQLite).
- Conflict test: two writers racing on `ExpectedVersion::Exact(0)` — one succeeds, one returns `EventStoreError::Conflict`.
- Load-from test: append 10 events, call `load_from(id, 5)` — assert 6 events returned (sequence 5–10 inclusive).
- `#[tokio::test]` for all async tests.
