//! Postgres datasource path coverage (Rule 95) — exercises the `sqlx-postgres`
//! backend without requiring a live server.

#![allow(clippy::unwrap_used, clippy::expect_used)]

/// @covers: connect — an unreachable Postgres host surfaces as `Connection`.
///
/// Port 59999 on loopback refuses fast, so this needs no running database.
#[cfg(feature = "postgres")]
#[tokio::test]
async fn test_connect_postgres_unreachable_host_returns_connection_error() {
    use swe_edge_egress_database_migration::{
        DatabaseConfig, DriverKind, MigrationError, MigrationSvc,
    };

    let cfg = DatabaseConfig {
        driver: DriverKind::Postgres,
        url: "postgres://postgres:postgres@127.0.0.1:59999/nonexistent".into(),
        max_connections: 1,
        acquire_timeout_secs: 5,
        idle_timeout_secs: None,
        migrations_dir: Some("./migrations".into()),
    };

    let err = MigrationSvc::connect(&cfg)
        .await
        .expect_err("connect to an unreachable host must fail");
    assert!(
        matches!(err, MigrationError::Connection(_)),
        "expected Connection error, got: {err}",
    );
}

/// Without the `postgres` feature the crate still builds; the noop runner works.
#[cfg(not(feature = "postgres"))]
#[tokio::test]
async fn test_postgres_feature_absent_noop_runner_still_works() {
    use swe_edge_egress_database_migration::{MigrationRunner, MigrationSvc};

    let runner = MigrationSvc::noop_migration_runner();
    let applied = runner.run().await.expect("noop run must succeed");
    assert!(applied.is_empty());
}
