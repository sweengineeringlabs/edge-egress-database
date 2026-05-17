//! Integration tests for the migration runner public API.

use swe_edge_egress_database_migration::{noop_migration_runner, MigrationError, MigrationRunner};

// ── noop runner (no features required) ───────────────────────────────────────

/// @covers: noop_migration_runner — run returns empty list.
#[tokio::test]
async fn test_noop_migration_runner_run_returns_empty_list() {
    let runner = noop_migration_runner();
    let applied = runner.run().await.expect("noop run must succeed");
    assert!(applied.is_empty(), "noop runner must apply no migrations");
}

/// @covers: noop_migration_runner — status returns empty list.
#[tokio::test]
async fn test_noop_migration_runner_status_returns_empty_list() {
    let runner = noop_migration_runner();
    let statuses = runner.status().await.expect("noop status must succeed");
    assert!(statuses.is_empty(), "noop runner has no known migrations");
}

/// @covers: noop_migration_runner — revert returns NoMigrationToRevert.
#[tokio::test]
async fn test_noop_migration_runner_revert_returns_no_migration_to_revert_error() {
    let runner = noop_migration_runner();
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
    let runner: Arc<dyn MigrationRunner> = Arc::new(noop_migration_runner());
    drop(runner);
}

/// @covers: noop_migration_runner — run is idempotent.
#[tokio::test]
async fn test_noop_migration_runner_run_is_idempotent() {
    let runner = noop_migration_runner();
    let first = runner.run().await.expect("first run");
    let second = runner.run().await.expect("second run");
    assert_eq!(first.len(), second.len());
}

// ── sqlite runner (requires `sqlite` feature) ────────────────────────────────
//
// Tests use `sqlite::memory:` to avoid platform-specific file path
// formatting.  The runner detects `:memory:` and uses max_connections(1)
// so all method calls share the same in-memory database.

/// @covers: migration_runner — connects to in-memory SQLite successfully.
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_sqlite_migration_runner_connects_to_in_memory_db() {
    use swe_edge_egress_database_migration::migration_runner;
    use std::fs;

    let dir = tempfile::tempdir().expect("temp dir");
    let migrations_path = dir.path().join("migrations");
    fs::create_dir(&migrations_path).unwrap();
    fs::write(migrations_path.join("1__init.sql"), "CREATE TABLE t (id INTEGER PRIMARY KEY);")
        .unwrap();

    migration_runner("sqlite::memory:", migrations_path.to_str().unwrap())
        .await
        .expect("connection to in-memory SQLite must succeed");
}

/// @covers: migration_runner — applies migrations and reports them as applied in status.
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_sqlite_migration_runner_applies_migrations_and_status_reflects_applied() {
    use swe_edge_egress_database_migration::migration_runner;
    use std::fs;

    let dir = tempfile::tempdir().expect("temp dir");
    let migrations_path = dir.path().join("migrations");
    fs::create_dir(&migrations_path).unwrap();
    fs::write(
        migrations_path.join("1__create_users.sql"),
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
    )
    .unwrap();
    fs::write(
        migrations_path.join("2__add_email.sql"),
        "ALTER TABLE users ADD COLUMN email TEXT;",
    )
    .unwrap();

    let runner = migration_runner("sqlite::memory:", migrations_path.to_str().unwrap())
        .await
        .expect("connect");

    // run() applies both pending migrations.
    let applied = runner.run().await.expect("first run must succeed");
    assert_eq!(applied.len(), 2, "both migrations must be applied on first run");
    assert_eq!(applied[0].version, 1);
    assert_eq!(applied[1].version, 2);

    // run() is idempotent — nothing left.
    let second = runner.run().await.expect("second run must succeed");
    assert!(second.is_empty(), "no pending migrations on second run");

    // status() reports both as applied.
    let statuses = runner.status().await.expect("status must succeed");
    assert_eq!(statuses.len(), 2);
    assert!(statuses.iter().all(|s| s.applied), "all migrations must show as applied");
}

/// @covers: migration_runner — MigrationsDirectoryNotFound for missing dir.
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_sqlite_migration_runner_run_returns_error_for_missing_migrations_dir() {
    use swe_edge_egress_database_migration::migration_runner;

    let runner = migration_runner("sqlite::memory:", "/nonexistent/__swe_edge_migrations__")
        .await
        .expect("connect must succeed even with missing migrations dir");

    let err = runner.run().await.expect_err("run with missing dir must fail");
    assert!(
        matches!(err, MigrationError::MigrationsDirectoryNotFound(_)),
        "expected MigrationsDirectoryNotFound, got: {err}",
    );
}

/// @covers: migration_runner — Connection error for unregistered URL scheme.
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_migration_runner_returns_connection_error_for_bad_url() {
    use swe_edge_egress_database_migration::migration_runner;

    match migration_runner("bad://invalid", "./migrations").await {
        Err(MigrationError::Connection(_)) => {}
        Err(other) => panic!("expected Connection error, got: {other}"),
        Ok(_) => panic!("expected error, got Ok"),
    }
}
