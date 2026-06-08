//! `swe-edge-egress-database-migration` — database datasource + migration bootstrap.
//!
//! A domain-free infrastructure leaf: it owns the `[database]` config contract
//! ([`DatabaseConfig`]), runs pending schema migrations via **refinery**
//! (RUSTSEC-2023-0071-clean; no rsa dependency), and returns a ready
//! **deadpool** [`DbPool`] the consumer queries through. The DB-backed
//! `edge_domain::Repository` adapters live in the consumer, not here (ADR-001).
//!
//! | Feature    | Driver     | URL scheme     |
//! |------------|------------|----------------|
//! _(none)_    | —          | — (noop only)  |
//! `sqlite`    | SQLite     | `sqlite:///…`  |
//! `postgres`  | PostgreSQL | `postgres://…` |
//!
//! Migration files follow **refinery** naming: `V{n}__{description}.sql`
//! (e.g. `V1__create_metrics.sql`).
//!
//! # Quick start — no-op (always available)
//!
//! ```rust
//! use swe_edge_egress_database_migration::{MigrationRunner, MigrationSvc};
//!
//! # #[tokio::main]
//! # async fn main() {
//! let runner = MigrationSvc::noop_migration_runner();
//! let applied = runner.run().await.unwrap();
//! assert!(applied.is_empty());
//! # }
//! ```
//!
//! # Quick start — connect + migrate (with the `sqlite` feature)
//!
//! ```rust,no_run
//! # #[cfg(feature = "sqlite")]
//! # async fn example() -> Result<(), swe_edge_egress_database_migration::MigrationError> {
//! use swe_edge_egress_database_migration::{DatabaseConfig, DriverKind, MigrationSvc};
//!
//! let cfg = DatabaseConfig {
//!     driver: DriverKind::Sqlite,
//!     url: "sqlite:///./obsrv.db".into(),
//!     max_connections: 5,
//!     acquire_timeout_secs: 30,
//!     idle_timeout_secs: None,
//!     migrations_dir: Some("./migrations".into()),
//! };
//!
//! let pool = MigrationSvc::connect_and_migrate(&cfg).await?;
//! // `pool.as_sqlite()` yields the deadpool-sqlite pool for the consumer's
//! // Repository adapters. Use `pool.get().await` to borrow a connection.
//! # let _ = pool;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![deny(unsafe_code)]
// `unwrap`/`expect` are denied in production code but are the idiomatic
// assertion mechanism in inline `#[cfg(test)]` modules.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod api;
mod core;
mod saf;
mod spi;

mod gateway;
pub use gateway::*;
