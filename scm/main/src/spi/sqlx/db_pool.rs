//! [`DbPool`] — concrete `sqlx`-backed connection pool returned by the datasource.
//!
//! This is an `spi/` type (it names the `sqlx` technology) surfaced to consumers
//! through `saf/`, per ADR-008. The neutral `api/` surface never references it.

/// A ready, configured connection pool with migrations applied.
///
/// Returned by [`MigrationSvc::connect_and_migrate`](crate::MigrationSvc::connect_and_migrate)
/// and [`MigrationSvc::connect`](crate::MigrationSvc::connect). Consumers query
/// through the concrete `sqlx` pool obtained from [`DbPool::as_sqlite`] /
/// [`DbPool::as_postgres`] to implement their `edge_domain::Repository` adapters.
///
/// The set of variants is feature-gated: only drivers whose cargo feature is
/// enabled exist.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum DbPool {
    /// A SQLite connection pool. Present with the `sqlite` feature.
    #[cfg(feature = "sqlite")]
    Sqlite(::sqlx::SqlitePool),
    /// A PostgreSQL connection pool. Present with the `postgres` feature.
    #[cfg(feature = "postgres")]
    Postgres(::sqlx::PgPool),
}

impl DbPool {
    /// Borrow the underlying SQLite pool, or `None` if this is a different driver.
    #[cfg(feature = "sqlite")]
    pub fn as_sqlite(&self) -> Option<&::sqlx::SqlitePool> {
        #[allow(irrefutable_let_patterns)]
        if let DbPool::Sqlite(pool) = self {
            Some(pool)
        } else {
            None
        }
    }

    /// Borrow the underlying PostgreSQL pool, or `None` if this is a different driver.
    #[cfg(feature = "postgres")]
    pub fn as_postgres(&self) -> Option<&::sqlx::PgPool> {
        #[allow(irrefutable_let_patterns)]
        if let DbPool::Postgres(pool) = self {
            Some(pool)
        } else {
            None
        }
    }

    /// Close the pool, waiting for in-flight connections to be released.
    ///
    /// Cloned handles to the same pool are closed too — `sqlx` pools share state.
    pub async fn close(&self) {
        match self {
            #[cfg(feature = "sqlite")]
            DbPool::Sqlite(pool) => pool.close().await,
            #[cfg(feature = "postgres")]
            DbPool::Postgres(pool) => pool.close().await,
        }
    }
}
