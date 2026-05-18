//! `swe-edge-egress-database-migration` — database migration runner.
//!
//! Provides the [`MigrationRunner`] trait backed by refinery.  Each database
//! driver is an independent optional feature — enabling `postgres` never pulls
//! in SQLite code and vice versa.
//!
//! | Feature    | Driver     | URL scheme       |
//! |------------|------------|------------------|
//! _(none)_    | —          | —                |
//! `postgres`  | PostgreSQL | `postgres://…`   |
//! `sqlite`    | SQLite     | `sqlite:///…`    |
//!
//! Migration files must follow refinery naming: `V{n}__{description}.sql`
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
//! let runner = migration_runner("sqlite:///./dev.db", "./migrations").await?;
//! let applied = runner.run().await?;
//! println!("applied {} migration(s)", applied.len());
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![deny(unsafe_code)]

mod api;
mod core;
mod saf;

pub use saf::*;
