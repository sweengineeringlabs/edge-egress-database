//! End-to-end tests for `MigrationSvc::connect_and_migrate` / `migration_runner`
//! against a real SQLite database (tempfile-backed).
//!
//! Migration files use sqlx naming: `<version>_<description>.sql`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[cfg(feature = "sqlite")]
use swe_edge_egress_database_migration::{
    DatabaseConfig, DriverKind, MigrationError, MigrationSvc,
};

#[cfg(feature = "sqlite")]
fn sqlite_url(path: &std::path::Path) -> String {
    // sqlx parses sqlite URLs via `url::Url`; use forward slashes so a Windows
    // path (with `\` and a drive letter) round-trips cleanly.
    format!("sqlite:///{}", path.to_str().unwrap().replace('\\', "/"))
}

#[cfg(feature = "sqlite")]
fn write_two_migrations(dir: &std::path::Path) {
    std::fs::write(
        dir.join("0001_create_metrics.sql"),
        "CREATE TABLE metrics (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
    )
    .unwrap();
    std::fs::write(
        dir.join("0002_add_value.sql"),
        "ALTER TABLE metrics ADD COLUMN value REAL NOT NULL DEFAULT 0.0;",
    )
    .unwrap();
}

#[cfg(feature = "sqlite")]
fn config_for(db: &std::path::Path, migrations_dir: Option<String>) -> DatabaseConfig {
    DatabaseConfig {
        driver: DriverKind::Sqlite,
        url: sqlite_url(db),
        max_connections: 4,
        acquire_timeout_secs: 30,
        idle_timeout_secs: None,
        migrations_dir,
    }
}

/// @covers: connect_and_migrate — returns a usable pool with the migrated schema.
///
/// Proves the schema actually exists by inserting and reading a row through the
/// returned pool (the exact path a consumer's Repository adapter takes).
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_connect_and_migrate_returns_pool_with_migrated_schema() {
    let work = tempfile::tempdir().expect("temp dir");
    let migs = work.path().join("migrations");
    std::fs::create_dir(&migs).unwrap();
    write_two_migrations(&migs);
    let db = work.path().join("obsrv.db");

    let cfg = config_for(&db, Some(migs.to_str().unwrap().to_string()));
    let pool = MigrationSvc::connect_and_migrate(&cfg)
        .await
        .expect("connect_and_migrate must succeed");

    let sqlite = pool.as_sqlite().expect("sqlite pool must be present");

    // The `value` column only exists if BOTH migrations applied in order.
    sqlx::query("INSERT INTO metrics (name, value) VALUES (?1, ?2)")
        .bind("cpu")
        .bind(0.91_f64)
        .execute(sqlite)
        .await
        .expect("insert into migrated table must succeed");

    let name: String = sqlx::query_scalar("SELECT name FROM metrics WHERE id = 1")
        .fetch_one(sqlite)
        .await
        .expect("select from migrated table must succeed");
    assert_eq!(name, "cpu");

    pool.close().await;
}

/// @covers: migration_runner — first run applies all; second run is idempotent;
/// status reflects applied.
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_migration_runner_run_is_idempotent_and_status_reflects_applied() {
    use swe_edge_egress_database_migration::MigrationRunner;

    let work = tempfile::tempdir().expect("temp dir");
    let migs = work.path().join("migrations");
    std::fs::create_dir(&migs).unwrap();
    write_two_migrations(&migs);
    let db = work.path().join("obsrv.db");

    let cfg = config_for(&db, Some(migs.to_str().unwrap().to_string()));
    let runner = MigrationSvc::migration_runner(&cfg)
        .await
        .expect("runner construction must succeed");

    let first = runner.run().await.expect("first run");
    assert_eq!(first.len(), 2, "both migrations applied on first run");
    assert_eq!(first[0].version, 1);
    assert_eq!(first[1].version, 2);

    let second = runner.run().await.expect("second run");
    assert!(
        second.is_empty(),
        "idempotent: second run applies nothing, got {second:?}"
    );

    let status = runner.status().await.expect("status");
    assert_eq!(status.len(), 2);
    assert!(
        status.iter().all(|s| s.applied),
        "all migrations must report applied"
    );
}

/// @covers: connect_and_migrate — missing `migrations_dir` is NotConfigured.
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_connect_and_migrate_without_migrations_dir_returns_not_configured() {
    let work = tempfile::tempdir().expect("temp dir");
    let db = work.path().join("obsrv.db");
    let cfg = config_for(&db, None);

    let err = MigrationSvc::connect_and_migrate(&cfg)
        .await
        .expect_err("connect_and_migrate without migrations_dir must fail");
    assert!(
        matches!(err, MigrationError::NotConfigured(_)),
        "expected NotConfigured, got: {err}",
    );
}

/// @covers: run — a non-existent migrations directory is MigrationsDirectoryNotFound.
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_run_with_absent_migrations_dir_returns_directory_not_found() {
    use swe_edge_egress_database_migration::MigrationRunner;

    let work = tempfile::tempdir().expect("temp dir");
    let db = work.path().join("obsrv.db");
    let missing = work.path().join("does_not_exist");

    let cfg = config_for(&db, Some(missing.to_str().unwrap().to_string()));
    let runner = MigrationSvc::migration_runner(&cfg)
        .await
        .expect("connect succeeds; dir is only read on run()");

    let err = runner
        .run()
        .await
        .expect_err("run against an absent migrations dir must fail");
    assert!(
        matches!(err, MigrationError::MigrationsDirectoryNotFound(_)),
        "expected MigrationsDirectoryNotFound, got: {err}",
    );
}

/// @covers: revert — the sqlx runner is forward-only.
#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_revert_returns_not_configured_forward_only() {
    use swe_edge_egress_database_migration::MigrationRunner;

    let work = tempfile::tempdir().expect("temp dir");
    let migs = work.path().join("migrations");
    std::fs::create_dir(&migs).unwrap();
    write_two_migrations(&migs);
    let db = work.path().join("obsrv.db");

    let cfg = config_for(&db, Some(migs.to_str().unwrap().to_string()));
    let runner = MigrationSvc::migration_runner(&cfg).await.expect("runner");

    let err = runner.revert().await.expect_err("revert must fail");
    assert!(
        matches!(err, MigrationError::NotConfigured(_)),
        "expected NotConfigured, got: {err}",
    );
}
