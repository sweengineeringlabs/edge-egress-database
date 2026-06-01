//! Integration tests for the migration runner public API.

use swe_edge_egress_database_migration::{MigrationError, MigrationRunner, MigrationSvc};

// ── noop runner (no features required) ───────────────────────────────────────

/// @covers: noop_migration_runner — run returns empty list.
#[tokio::test]
async fn test_noop_migration_runner_run_returns_empty_list() {
    let runner = MigrationSvc::noop_migration_runner();
    let applied = runner.run().await.expect("noop run must succeed");
    assert!(applied.is_empty(), "noop runner must apply no migrations");
}

/// @covers: noop_migration_runner — status returns empty list.
#[tokio::test]
async fn test_noop_migration_runner_status_returns_empty_list() {
    let runner = MigrationSvc::noop_migration_runner();
    let statuses = runner.status().await.expect("noop status must succeed");
    assert!(statuses.is_empty(), "noop runner has no known migrations");
}

/// @covers: noop_migration_runner — revert returns NoMigrationToRevert.
#[tokio::test]
async fn test_noop_migration_runner_revert_returns_no_migration_to_revert_error() {
    let runner = MigrationSvc::noop_migration_runner();
    let err = runner.revert().await.expect_err("noop revert must fail");
    assert!(
        matches!(err, MigrationError::NoMigrationToRevert),
        "expected NoMigrationToRevert, got: {err}",
    );
}

/// @covers: MigrationRunner — object-safe; storable as Arc<dyn MigrationRunner>.
#[test]
fn test_migration_runner_can_be_stored_as_arc_dyn_trait() {
    use std::sync::Arc;
    let runner: Arc<dyn MigrationRunner> = Arc::new(MigrationSvc::noop_migration_runner());
    drop(runner);
}

/// @covers: noop_migration_runner — run is idempotent.
#[tokio::test]
async fn test_noop_migration_runner_run_is_idempotent() {
    let runner = MigrationSvc::noop_migration_runner();
    let first = runner.run().await.expect("first run");
    let second = runner.run().await.expect("second run");
    assert_eq!(first.len(), second.len());
}

// ── sqlite runner (requires `sqlite` feature) ────────────────────────────────
//
// Tests use a tempfile-backed SQLite database.  In-memory SQLite is not
// supported — each rusqlite connection opens a fresh database, so state
// would not persist between run() and status() calls.
//
// Migration files follow refinery naming: V{n}__{description}.sql

/// @covers: migration_runner — connects to a SQLite file successfully.
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_sqlite_migration_runner_connects_to_sqlite_file() {
    use std::fs;

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

/// @covers: migration_runner — applies migrations and status reflects applied.
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_sqlite_migration_runner_applies_migrations_and_status_reflects_applied() {
    use std::fs;

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

/// @covers: migration_runner — MigrationsDirectoryNotFound for missing dir.
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_sqlite_migration_runner_run_returns_error_for_missing_migrations_dir() {
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

/// @covers: migration_runner — NotConfigured for unrecognised URL scheme.
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_migration_runner_returns_not_configured_for_unknown_url_scheme() {
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
