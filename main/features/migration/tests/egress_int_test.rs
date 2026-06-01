//! Tests exercising the SPI egress extension point.
//!
//! The `spi::egress` module is the SEA extension anchor — its presence signals
//! that downstream consumers may substitute custom [`MigrationRunner`] implementations.

use swe_edge_egress_database_migration::MigrationRunner;

/// Verify the [`MigrationRunner`] trait can be implemented by a downstream consumer.
///
/// This test exercises the extension contract that `spi::egress` advertises.
struct CustomRunner;

impl MigrationRunner for CustomRunner {
    fn run(
        &self,
    ) -> futures::future::BoxFuture<
        '_,
        Result<
            Vec<swe_edge_egress_database_migration::Migration>,
            swe_edge_egress_database_migration::MigrationError,
        >,
    > {
        Box::pin(async { Ok(vec![]) })
    }

    fn revert(
        &self,
    ) -> futures::future::BoxFuture<
        '_,
        Result<
            swe_edge_egress_database_migration::Migration,
            swe_edge_egress_database_migration::MigrationError,
        >,
    > {
        Box::pin(async {
            Err(swe_edge_egress_database_migration::MigrationError::NoMigrationToRevert)
        })
    }

    fn status(
        &self,
    ) -> futures::future::BoxFuture<
        '_,
        Result<
            Vec<swe_edge_egress_database_migration::MigrationStatus>,
            swe_edge_egress_database_migration::MigrationError,
        >,
    > {
        Box::pin(async { Ok(vec![]) })
    }
}

#[test]
fn test_egress_spi_custom_runner_implements_migration_runner() {
    fn _accept(_: &dyn MigrationRunner) {}
    let runner = CustomRunner;
    _accept(&runner);
}

#[tokio::test]
async fn test_egress_spi_custom_runner_run_returns_empty() {
    let runner = CustomRunner;
    let result = runner.run().await.expect("custom runner run must succeed");
    assert!(result.is_empty());
}
