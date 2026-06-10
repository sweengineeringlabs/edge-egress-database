//! `deadpool`-backed datasource implementation (external library — ADR-008 `spi/`).
//!
//! Holds the only `deadpool`-aware code in the crate: the connect/ping
//! datasource. The [`DbPool`] value object lives in `api/types/` (SEA Rule 160).
//! Migrations are run by `core::refinery` on a fresh connection before the pool
//! is returned. Surfaced to consumers exclusively through `saf/`.

pub(crate) mod datasource;

pub(crate) use datasource::DeadpoolDatasource;
