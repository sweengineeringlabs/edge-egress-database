//! `swe-edge-egress-database-migration` — database-agnostic migration runner.
//!
//! Provides the [`MigrationRunner`] trait and a single sqlx `AnyPool`-backed
//! implementation. The database backend is selected at runtime from the
//! connection URL — no recompilation needed to switch from SQLite in dev to
//! PostgreSQL in production.
//!
//! | Feature    | Driver compiled in | URL scheme     |
//! |------------|--------------------|----------------|
//! _(none)_    | —                  | —              |
//! `postgres`  | PostgreSQL         | `postgres://…` |
//! `sqlite`    | SQLite             | `sqlite://…`   |
//! `mysql`     | MySQL              | `mysql://…`    |
//!
//! Multiple features may be active simultaneously.
//!
//! # Quick start — no-op (always available)
//!
//! ```rust
//! use swe_edge_egress_database_migration::{noop_migration_runner, MigrationRunner};
//!
//! # #[tokio::main]
//! # async fn main() {
//! let runner = noop_migration_runner();
//! let applied = runner.run().await.unwrap();
//! assert!(applied.is_empty());
//! # }
//! ```
//!
//! # Quick start — SQLite (with `sqlite` feature)
//!
//! ```rust,no_run
//! # #[cfg(feature = "sqlite")]
//! # async fn example() -> Result<(), swe_edge_egress_database_migration::MigrationError> {
//! use swe_edge_egress_database_migration::{migration_runner, MigrationRunner};
//!
//! // URL scheme selects the backend at runtime — switch to postgres:// for PostgreSQL.
//! let runner = migration_runner("sqlite::memory:", "./migrations").await?;
//! let applied = runner.run().await?;
//! println!("applied {} migration(s)", applied.len());
//!
//! let status = runner.status().await?;
//! for s in &status {
//!     println!("v{} {} — {}", s.migration.version, s.migration.description,
//!         if s.applied { "applied" } else { "pending" });
//! }
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![deny(unsafe_code)]

mod api;
mod core;
mod saf;

pub use saf::*;
