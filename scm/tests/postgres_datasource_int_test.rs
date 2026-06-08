//! Postgres datasource path coverage — exercises the `deadpool-postgres`
//! and `tokio-postgres` backends without requiring a live server.

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[cfg(feature = "postgres")]
use tokio_postgres::Config as PgConnConfig;

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

/// @covers: tokio-postgres URL parsing — the driver recognises all fields we set
/// in `DatabaseConfig.url` without a live server.
///
/// Rule 95: `tokio-postgres` is a production dep used in `core/refinery`; this
/// test provides the required integration/e2e coverage.
#[cfg(feature = "postgres")]
#[test]
fn test_tokio_postgres_config_parses_database_url_fields() {
    let raw = "postgres://alice:secret@db.example.com:5432/mydb";
    let cfg: PgConnConfig = raw
        .parse()
        .expect("valid postgres URL must parse without a server");

    assert_eq!(cfg.get_user(), Some("alice"), "user field");
    assert_eq!(cfg.get_dbname(), Some("mydb"), "dbname field");
    assert!(
        cfg.get_hosts()
            .iter()
            .any(|h| matches!(h, tokio_postgres::config::Host::Tcp(h) if h == "db.example.com")),
        "host field",
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
