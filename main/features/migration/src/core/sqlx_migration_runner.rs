//! sqlx-backed migration runner using a shared `AnyPool`.
//!
//! The pool is created once at construction; all operations reuse it.
//! This means in-memory SQLite (`sqlite::memory:`) works correctly —
//! every call sees the same database rather than a fresh one.
//!
//! The backend is selected at runtime by the URL scheme:
//!
//! | URL prefix      | Driver     | Feature |
//! |-----------------|------------|---------|
//! | `postgres://…`  | PostgreSQL | `postgres` |
//! | `sqlite://…`    | SQLite     | `sqlite`   |
//! | `sqlite::memory:` | SQLite in-memory | `sqlite` |
//! | `mysql://…`     | MySQL      | `mysql`    |

use std::collections::HashSet;
use std::path::Path;

use futures::future::BoxFuture;
use sqlx::migrate::Migrator;
use sqlx::pool::PoolOptions;
use sqlx::{Any, AnyPool};

use crate::api::migration::{Migration, MigrationStatus};
use crate::api::migration_error::MigrationError;
use crate::api::migration_runner::MigrationRunner;

/// sqlx `AnyPool`-backed migration runner.
///
/// Construct via the async [`crate::migration_runner`] SAF factory.
/// Do not name this type in consumer code.
pub(crate) struct SqlxMigrationRunner {
    pool: AnyPool,
    migrations_dir: String,
}

impl SqlxMigrationRunner {
    /// Connect to the database and return a ready runner.
    ///
    /// In-memory SQLite (`sqlite::memory:`) is automatically configured
    /// with `max_connections(1)` so all operations share the same database.
    pub(crate) async fn connect(
        url: impl Into<String>,
        migrations_dir: impl Into<String>,
    ) -> Result<Self, MigrationError> {
        sqlx::any::install_default_drivers();
        let url = url.into();
        // In-memory SQLite requires a single shared connection — otherwise
        // each checkout gets a fresh empty database.
        let max_connections = if url.contains(":memory:") { 1 } else { 5 };
        let pool: AnyPool = PoolOptions::<Any>::new()
            .max_connections(max_connections)
            .connect(&url)
            .await
            .map_err(|e| MigrationError::Connection(e.to_string()))?;
        Ok(Self { pool, migrations_dir: migrations_dir.into() })
    }

    async fn migrator(&self) -> Result<Migrator, MigrationError> {
        Migrator::new(Path::new(&self.migrations_dir))
            .await
            .map_err(|e| MigrationError::MigrationsDirectoryNotFound(e.to_string()))
    }
}

impl MigrationRunner for SqlxMigrationRunner {
    fn run(&self) -> BoxFuture<'_, Result<Vec<Migration>, MigrationError>> {
        Box::pin(async move {
            let migrator = self.migrator().await?;
            let before = applied_versions(&self.pool).await;

            migrator
                .run(&self.pool)
                .await
                .map_err(|e| MigrationError::Apply { version: 0, reason: e.to_string() })?;

            let after = applied_versions(&self.pool).await;
            let newly_applied = migrator
                .migrations
                .iter()
                .filter(|m| {
                    !m.migration_type.is_down_migration()
                        && !before.contains(&m.version)
                        && after.contains(&m.version)
                })
                .map(|m| Migration::pending(m.version, m.description.as_ref()))
                .collect();

            Ok(newly_applied)
        })
    }

    fn revert(&self) -> BoxFuture<'_, Result<Migration, MigrationError>> {
        Box::pin(async move {
            let migrator = self.migrator().await?;
            let before = applied_versions(&self.pool).await;
            if before.is_empty() {
                return Err(MigrationError::NoMigrationToRevert);
            }
            let target = *before.iter().max().unwrap();

            migrator
                .undo(&self.pool, 1)
                .await
                .map_err(|e| MigrationError::Revert { version: target, reason: e.to_string() })?;

            let description = migrator
                .migrations
                .iter()
                .find(|m| m.version == target)
                .map(|m| m.description.to_string())
                .unwrap_or_default();

            Ok(Migration::pending(target, description))
        })
    }

    fn status(&self) -> BoxFuture<'_, Result<Vec<MigrationStatus>, MigrationError>> {
        Box::pin(async move {
            let migrator = self.migrator().await?;
            let applied = applied_versions(&self.pool).await;

            let statuses = migrator
                .migrations
                .iter()
                .filter(|m| !m.migration_type.is_down_migration())
                .map(|m| MigrationStatus {
                    migration: Migration::pending(m.version, m.description.as_ref()),
                    applied: applied.contains(&m.version),
                })
                .collect();

            Ok(statuses)
        })
    }
}

async fn applied_versions(pool: &AnyPool) -> HashSet<i64> {
    sqlx::query_scalar::<_, i64>("SELECT version FROM _sqlx_migrations")
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .collect()
}
