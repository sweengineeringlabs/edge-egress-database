//! `deadpool`-backed datasource implementation (external library — ADR-008 `spi/`).
//!
//! Holds the only `deadpool`-aware code in the crate: the [`DbPool`] type and
//! the connect/ping datasource. Migrations are run by `core::refinery` on a
//! fresh connection before the pool is returned. Surfaced to consumers
//! exclusively through `saf/`.

pub(crate) mod datasource;
pub(crate) mod db_pool;

pub(crate) use datasource::DeadpoolDatasource;
pub use db_pool::DbPool;
