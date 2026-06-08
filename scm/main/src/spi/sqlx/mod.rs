//! `sqlx`-backed datasource implementation (external library — ADR-008 `spi/`).
//!
//! Holds the only `sqlx`-aware code in the crate: the [`DbPool`] type, the
//! connect/migrate datasource functions, and the [`SqlxMigrationRunner`].
//! Surfaced to consumers exclusively through `saf/`.

pub(crate) mod datasource;
pub(crate) mod db_pool;
pub(crate) mod sqlx_migration_runner;

pub use db_pool::DbPool;
pub(crate) use sqlx_migration_runner::SqlxMigrationRunner;
