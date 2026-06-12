# ADR-002: Repository Implementation — egress/database

**Status:** Accepted  
**Date:** 2026-06-12  
**Governing ADR:** [ADR-019](https://github.com/sweengineeringlabs/edge/blob/main/docs/3-architecture/adr/ADR-019-repository-pattern.md) — Repository Pattern

---

## Mandate

Implement `Repository<T, Id>` and `QueryableRepository<T, Id>` from `edge-domain-repository` for each database backend supported by `egress/database`.

---

## Dependency

```toml
edge-domain-repository = { git = "https://github.com/sweengineeringlabs/edge-domain" }
```

---

## Backends

| Backend | Feature flag |
|---|---|
| Postgres | `postgres` |
| SQLite | `sqlite` |
| In-memory | always-on (`RepositoryFactory::in_memory()`) |

---

## Required methods

`find`, `save`, `delete`, `list` are the four required methods. Provided default methods (`exists`, `count`, `list_page`) are inherited from the trait; they SHOULD be overridden for backends where a `COUNT(*)` or `LIMIT/OFFSET` query is more efficient than loading the full list.

---

## `QueryableRepository` query push-down

Database implementations MUST override `find_by` and `find_one_by` with backend-native queries rather than relying on the default in-process filter. The `Spec<T>` predicate is translated by a per-backend `SpecTranslator` registered at construction.

---

## `RepositoryError` mapping

| Backend error | `RepositoryError` variant |
|---|---|
| Row not found | `find` returns `Ok(None)` — not an error |
| Unique-key violation | `Conflict` |
| Connection failure | `Unavailable` |
| Unexpected DB error | `Internal(message)` |

---

## Testing

- Unit tests use `RepositoryFactory::in_memory()` — no backend dep.
- Integration tests use a per-test table prefix.
- Conflict test: `save` the same `id` twice, assert second call succeeds (upsert) or returns `Conflict` depending on implementation contract.
- Spec push-down test: insert 100 rows, `find_by` with a spec that matches 10 — assert exactly 10 returned, query plan shows no full scan (integration only).
- `#[tokio::test]` for all async tests.
