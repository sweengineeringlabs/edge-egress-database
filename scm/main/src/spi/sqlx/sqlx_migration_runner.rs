//! [`SqlxMigrationRunner`] — `sqlx`-backed implementation of [`MigrationRunner`].
//!
//! Holds a ready [`DbPool`] and a migrations directory, delegating to the
//! [`datasource`](crate::spi::sqlx::datasource) helpers. Forward-only: `revert`
//! returns [`MigrationError::NotConfigured`].

use futures::future::BoxFuture;

use crate::api::error::MigrationError;
use crate::api::migration::{Migration, MigrationStatus};
use crate::api::traits::MigrationRunner;
use crate::spi::sqlx::datasource;
use crate::spi::sqlx::db_pool::DbPool;

pub(crate) struct SqlxMigrationRunner {
    pool: DbPool,
    migrations_dir: String,
}

impl SqlxMigrationRunner {
    pub(crate) fn new(pool: DbPool, migrations_dir: String) -> Self {
        Self {
            pool,
            migrations_dir,
        }
    }
}

impl MigrationRunner for SqlxMigrationRunner {
    fn run(&self) -> BoxFuture<'_, Result<Vec<Migration>, MigrationError>> {
        Box::pin(async move { datasource::run_migrations(&self.pool, &self.migrations_dir).await })
    }

    fn revert(&self) -> BoxFuture<'_, Result<Migration, MigrationError>> {
        Box::pin(async {
            Err(MigrationError::NotConfigured(
                "revert is not supported by the sqlx runner (forward-only migrations)".into(),
            ))
        })
    }

    fn status(&self) -> BoxFuture<'_, Result<Vec<MigrationStatus>, MigrationError>> {
        Box::pin(
            async move { datasource::migration_status(&self.pool, &self.migrations_dir).await },
        )
    }
}
