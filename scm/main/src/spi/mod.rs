//! Extension hooks and external-library implementations for downstream consumers.
//!
//! `spi/egress` is the SEA extension anchor signalling that `saf/` returns
//! `impl MigrationRunner`. `spi/sqlx` holds the `sqlx`-backed datasource
//! (external library, surfaced via `saf/` per ADR-008).

pub(crate) mod egress;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
pub(crate) mod sqlx;
