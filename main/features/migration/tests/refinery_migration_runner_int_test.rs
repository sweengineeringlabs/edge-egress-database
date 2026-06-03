//! Integration tests for `RefineryMigrationRunner`.
//!
//! Tests cover the refinery-backed runner through the public SAF API,
//! exercising the SQLite backend (no live Postgres required).

#![allow(clippy::unwrap_used, clippy::expect_used)]

// ── sqlite backend (requires `sqlite` feature) ───────────────────────────────

/// @covers: MigrationSvc::migration_runner — connects to a SQLite file successfully.
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_sqlite_refinery_runner_connects_to_sqlite_file() {
    use std::fs;
    use swe_edge_egress_database_migration::MigrationSvc;

    let db_file = tempfile::NamedTempFile::new().expect("temp db file");
    let dir = tempfile::tempdir().expect("temp dir");
    let migs_path = dir.path().join("migrations");
    fs::create_dir(&migs_path).unwrap();
    fs::write(
        migs_path.join("V1__init.sql"),
        "CREATE TABLE t (id INTEGER PRIMARY KEY);",
    )
    .unwrap();

    let url = format!("sqlite:///{}", db_file.path().to_str().unwrap());
    MigrationSvc::migration_runner(&url, migs_path.to_str().unwrap())
        .await
        .expect("connection to SQLite file must succeed");
}

/// @covers: MigrationSvc::migration_runner — applies migrations, status reflects applied.
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_sqlite_refinery_runner_applies_migrations_and_status_reflects_applied() {
    use std::fs;
    use swe_edge_egress_database_migration::{MigrationRunner, MigrationSvc};

    let db_file = tempfile::NamedTempFile::new().expect("temp db file");
    let dir = tempfile::tempdir().expect("temp dir");
    let migs_path = dir.path().join("migrations");
    fs::create_dir(&migs_path).unwrap();
    fs::write(
        migs_path.join("V1__create_users.sql"),
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
    )
    .unwrap();
    fs::write(
        migs_path.join("V2__add_email.sql"),
        "ALTER TABLE users ADD COLUMN email TEXT;",
    )
    .unwrap();

    let url = format!("sqlite:///{}", db_file.path().to_str().unwrap());
    let runner = MigrationSvc::migration_runner(&url, migs_path.to_str().unwrap())
        .await
        .expect("connect");

    let applied = runner.run().await.expect("first run must succeed");
    assert_eq!(
        applied.len(),
        2,
        "both migrations must be applied on first run"
    );
    assert_eq!(applied[0].version, 1);
    assert_eq!(applied[1].version, 2);

    let second = runner.run().await.expect("second run must succeed");
    assert!(second.is_empty(), "no pending migrations on second run");

    let statuses = runner.status().await.expect("status must succeed");
    assert_eq!(statuses.len(), 2);
    assert!(
        statuses.iter().all(|s| s.applied),
        "all migrations must show as applied"
    );
}

/// @covers: MigrationSvc::migration_runner — MigrationsDirectoryNotFound for missing dir.
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_sqlite_refinery_runner_run_returns_error_for_missing_migrations_dir() {
    use swe_edge_egress_database_migration::{MigrationError, MigrationRunner, MigrationSvc};

    let db_file = tempfile::NamedTempFile::new().expect("temp db file");
    let url = format!("sqlite:///{}", db_file.path().to_str().unwrap());

    let runner = MigrationSvc::migration_runner(&url, "/nonexistent/__swe_edge_migrations__")
        .await
        .expect("connect must succeed even with missing migrations dir");

    let err = runner
        .run()
        .await
        .expect_err("run with missing dir must fail");
    assert!(
        matches!(err, MigrationError::MigrationsDirectoryNotFound(_)),
        "expected MigrationsDirectoryNotFound, got: {err}",
    );
}

/// @covers: MigrationSvc::migration_runner — NotConfigured for unknown URL scheme.
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_refinery_runner_returns_not_configured_for_unknown_url_scheme() {
    use swe_edge_egress_database_migration::{MigrationError, MigrationRunner, MigrationSvc};

    let runner = MigrationSvc::migration_runner("bad://invalid", "./migrations")
        .await
        .expect("factory must not fail — connection is lazy");

    let err = runner
        .run()
        .await
        .expect_err("run with unknown scheme must fail");
    assert!(
        matches!(err, MigrationError::NotConfigured(_)),
        "expected NotConfigured, got: {err}",
    );
}

/// @covers: MigrationSvc::migration_runner — revert returns NotConfigured.
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_refinery_runner_revert_returns_not_configured() {
    use swe_edge_egress_database_migration::{MigrationError, MigrationRunner, MigrationSvc};

    let db_file = tempfile::NamedTempFile::new().expect("temp db file");
    let url = format!("sqlite:///{}", db_file.path().to_str().unwrap());
    let runner = MigrationSvc::migration_runner(&url, "./migrations")
        .await
        .expect("connect");

    let err = runner.revert().await.expect_err("revert must fail");
    assert!(
        matches!(err, MigrationError::NotConfigured(_)),
        "expected NotConfigured, got: {err}",
    );
}

// ── postgres dependency coverage (Rule 95) ───────────────────────────────────
//
// tokio-postgres is used in src/ behind the `postgres` feature flag.
// These tests exercise the dependency at the type-system level; a live
// Postgres instance is not required in CI (the `postgres` feature is not
// enabled by default).

/// @covers: tokio-postgres dep — connection error surfaces as MigrationError::Connection.
#[cfg(feature = "postgres")]
#[tokio::test]
async fn test_postgres_refinery_runner_returns_connection_error_for_unreachable_host() {
    use swe_edge_egress_database_migration::{MigrationError, MigrationRunner, MigrationSvc};

    let runner = MigrationSvc::migration_runner(
        "postgres://postgres:postgres@127.0.0.1:59999/nonexistent",
        "./migrations",
    )
    .await
    .expect("factory must not fail — connection is lazy");

    let err = runner
        .run()
        .await
        .expect_err("run against unreachable host must fail");
    assert!(
        matches!(err, MigrationError::Connection(_)),
        "expected Connection error, got: {err}",
    );
}
