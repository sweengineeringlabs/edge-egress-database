//! Extension hooks for downstream consumers.
//!
//! The presence of `spi/` signals that `saf/` intentionally returns
//! `impl MigrationRunner` — consumers may substitute their own implementations
//! by implementing the [`MigrationRunner`] trait from `api/traits/`.

pub(crate) mod egress;
