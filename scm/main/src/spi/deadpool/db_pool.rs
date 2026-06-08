//! [`DbPool`] — deadpool-backed connection pool returned by the datasource.
//!
//! This is an `spi/` type (it names the `deadpool` technology) surfaced to
//! consumers through `saf/`, per ADR-008. The neutral `api/` surface never
//! references it. Migrations are run by `core::refinery` on a fresh connection;
//! this pool serves post-migration consumer queries only.

/// A ready, configured connection pool returned by
/// [`MigrationSvc::connect_and_migrate`](crate::MigrationSvc::connect_and_migrate)
/// and [`MigrationSvc::connect`](crate::MigrationSvc::connect).
///
/// Consumers call [`DbPool::as_sqlite`] / [`DbPool::as_postgres`] to obtain the
/// concrete deadpool pool for their `edge_domain::Repository` adapters.
/// Connections are checked out via `pool.get().await` and returned on drop.
///
/// The set of variants is feature-gated: only drivers whose cargo feature is
/// enabled exist at compile time.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum DbPool {
    /// A SQLite connection pool. Present with the `sqlite` feature.
    #[cfg(feature = "sqlite")]
    Sqlite(deadpool_sqlite::Pool),
    /// A PostgreSQL connection pool. Present with the `postgres` feature.
    #[cfg(feature = "postgres")]
    Postgres(deadpool_postgres::Pool),
}

impl DbPool {
    /// Borrow the underlying `deadpool_sqlite` pool, or `None` for other drivers.
    ///
    /// Use `pool.get().await` on the returned pool to acquire a connection, then
    /// `conn.interact(|c| { … }).await` to run rusqlite operations.
    #[cfg(feature = "sqlite")]
    pub fn as_sqlite(&self) -> Option<&deadpool_sqlite::Pool> {
        #[allow(irrefutable_let_patterns)]
        if let DbPool::Sqlite(pool) = self {
            Some(pool)
        } else {
            None
        }
    }

    /// Borrow the underlying `deadpool_postgres` pool, or `None` for other drivers.
    ///
    /// Use `pool.get().await` on the returned pool to acquire a
    /// `tokio_postgres::Client`-backed connection for async queries.
    #[cfg(feature = "postgres")]
    pub fn as_postgres(&self) -> Option<&deadpool_postgres::Pool> {
        #[allow(irrefutable_let_patterns)]
        if let DbPool::Postgres(pool) = self {
            Some(pool)
        } else {
            None
        }
    }

    /// Signal the pool to stop accepting new checkouts and drain idle connections.
    ///
    /// In-flight checkouts finish naturally; this call returns immediately after
    /// marking the pool closed. Cloned handles share state, so all clones are
    /// affected.
    pub async fn close(&self) {
        match self {
            #[cfg(feature = "sqlite")]
            DbPool::Sqlite(pool) => pool.close(),
            #[cfg(feature = "postgres")]
            DbPool::Postgres(pool) => pool.close(),
        }
    }
}
