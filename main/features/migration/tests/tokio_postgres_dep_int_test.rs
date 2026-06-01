//! Integration test coverage for the `tokio-postgres` dependency.
//!
//! Rule 95 requires that every dependency used in `src/` has integration or e2e test coverage.
//! The `tokio-postgres` crate is used in `src/core/refinery/refinery_migration_runner.rs`
//! behind the `postgres` feature flag.

/// @covers: tokio-postgres dep — connection attempt to unreachable host returns error.
///
/// This test exercises `tokio_postgres::connect` through the public SAF API.
#[cfg(feature = "postgres")]
#[tokio::test]
async fn test_tokio_postgres_dep_connection_error_surfaces_as_migration_error() {
    use swe_edge_egress_database_migration::{MigrationError, MigrationRunner, MigrationSvc};
    // tokio_postgres is exercised here via MigrationSvc::migration_runner
    // which calls tokio_postgres::connect under the hood.
    use tokio_postgres::NoTls;

    // Verify the tokio_postgres type is accessible (dep is correctly wired).
    let connect_result =
        tokio_postgres::connect("postgres://postgres:postgres@127.0.0.1:59999/test", NoTls).await;
    assert!(
        connect_result.is_err(),
        "unreachable host must produce error"
    );

    // Also verify the migration runner surfaces the error correctly.
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
        "expected Connection error wrapping tokio_postgres error, got: {err}",
    );
}

// When postgres feature is not enabled, provide a smoke test that confirms the
// crate compiles without the dep.
#[cfg(not(feature = "postgres"))]
#[test]
fn test_tokio_postgres_dep_not_enabled_crate_compiles_without_it() {
    use swe_edge_egress_database_migration::MigrationSvc;
    let _runner = MigrationSvc::noop_migration_runner();
}
