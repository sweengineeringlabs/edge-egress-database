//! Tests for the `DbPool` handle returned by the datasource.

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[cfg(feature = "sqlite")]
use swe_edge_egress_database_migration::{DatabaseConfig, DriverKind, MigrationSvc};

/// @covers: DbPool::as_sqlite — yields the concrete sqlite pool; close() works.
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_db_pool_as_sqlite_returns_concrete_pool_and_closes() {
    let work = tempfile::tempdir().expect("temp dir");
    let db = work.path().join("pool.db");
    let cfg = DatabaseConfig {
        driver: DriverKind::Sqlite,
        url: format!("sqlite:///{}", db.to_str().unwrap().replace('\\', "/")),
        max_connections: 2,
        acquire_timeout_secs: 30,
        idle_timeout_secs: Some(60),
        migrations_dir: None,
    };

    // `connect` opens a pool without running migrations.
    let pool = MigrationSvc::connect(&cfg)
        .await
        .expect("connect must open a pool");

    let sqlite = pool.as_sqlite().expect("as_sqlite must return the pool");
    // A trivial query proves the pool yields live connections.
    let one: i64 = sqlx::query_scalar("SELECT 1")
        .fetch_one(sqlite)
        .await
        .expect("SELECT 1 must succeed on a live pool");
    assert_eq!(one, 1);

    pool.close().await;
}
