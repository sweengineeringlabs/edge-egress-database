//! `swe-edge-egress-database-migration` — database migration runner.
//!
//! Provides the [`MigrationRunner`] trait and concrete implementations
//! behind feature flags:
//!
//! | Feature | Runner factory | Backend |
//! |---------|---------------|---------|
//! _(none)_ | [`noop_migration_runner`] | In-process no-op |
//! `postgres` | [`postgres_migration_runner`] | PostgreSQL via sqlx |
//! `sqlite` | [`sqlite_migration_runner`] | SQLite via sqlx |
//!
//! # Quick start
//!
//! ```rust,no_run
//! use swe_edge_egress_database_migration::{noop_migration_runner, MigrationRunner};
//!
//! # #[tokio::main]
//! # async fn main() {
//! let runner = noop_migration_runner();
//! let applied = runner.run().await.unwrap();
//! println!("applied: {}", applied.len());
//! # }
//! ```
//!
//! With the `postgres` feature:
//!
//! ```rust,no_run
//! # #[cfg(feature = "postgres")]
//! # async fn example() {
//! use swe_edge_egress_database_migration::postgres_migration_runner;
//!
//! let runner = postgres_migration_runner("postgres://localhost/mydb", "./migrations");
//! let applied = runner.run().await.unwrap();
//! # }
//! ```

#![warn(missing_docs)]
#![deny(unsafe_code)]

mod api;
mod core;
mod saf;

pub use saf::*;
