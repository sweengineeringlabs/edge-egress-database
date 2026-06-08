//! Extension hooks and external-library implementations for downstream consumers.
//!
//! `spi/egress` is the SEA extension anchor signalling that `saf/` returns
//! `impl MigrationRunner`. `spi/deadpool` holds the deadpool-backed pool
//! (external library, surfaced via `saf/` per ADR-008). Migrations are run by
//! `core::refinery` on a dedicated connection before the pool is returned.

pub(crate) mod egress;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
pub(crate) mod deadpool;
