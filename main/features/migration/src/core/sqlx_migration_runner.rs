//! sqlx-backed migration runner.
//!
//! Enabled by the `postgres`, `sqlite`, or `mysql` feature flags.
//! Uses `sqlx::migrate::Migrator` to apply and revert migrations loaded
//! from a directory at runtime.
//!
//! Migration files follow the sqlx naming convention:
//!   `V{version}__{description}.sql`       (up / apply)
//!   `V{version}__{description}.down.sql`  (down / revert, optional)

use std::collections::HashSet;
use std::path::Path;

use futures::future::BoxFuture;
use sqlx::migrate::Migrator;

use crate::api::migration::{Migration, MigrationStatus};
use crate::api::migration_error::MigrationError;
use crate::api::migration_runner::MigrationRunner;

/// Database URL and migrations directory for a sqlx-backed runner.
pub(crate) struct SqlxMigrationRunner {
    url: String,
    migrations_dir: String,
}

impl SqlxMigrationRunner {
    pub(crate) fn new(url: impl Into<String>, migrations_dir: impl Into<String>) -> Self {
        Self { url: url.into(), migrations_dir: migrations_dir.into() }
    }

    async fn migrator(&self) -> Result<Migrator, MigrationError> {
        Migrator::new(Path::new(&self.migrations_dir))
            .await
            .map_err(|e| MigrationError::MigrationsDirectoryNotFound(e.to_string()))
    }
}

// ── postgres ─────────────────────────────────────────────────────────────────

#[cfg(feature = "postgres")]
mod pg {
    use super::*;
    use sqlx::PgPool;

    impl MigrationRunner for SqlxMigrationRunner {
        fn run(&self) -> BoxFuture<'_, Result<Vec<Migration>, MigrationError>> {
            Box::pin(async move {
                let pool = PgPool::connect(&self.url)
                    .await
                    .map_err(|e| MigrationError::Connection(e.to_string()))?;
                let migrator = self.migrator().await?;

                // Determine which versions are already applied.
                let before: HashSet<i64> = applied_versions(&pool).await;

                migrator
                    .run(&pool)
                    .await
                    .map_err(|e| MigrationError::Apply { version: 0, reason: e.to_string() })?;

                let after: HashSet<i64> = applied_versions(&pool).await;
                let newly_applied = migrator
                    .migrations
                    .iter()
                    .filter(|m| !m.migration_type.is_down_migration()
                        && !before.contains(&m.version)
                        && after.contains(&m.version))
                    .map(|m| Migration::pending(m.version, m.description.as_ref()))
                    .collect();

                Ok(newly_applied)
            })
        }

        fn revert(&self) -> BoxFuture<'_, Result<Migration, MigrationError>> {
            Box::pin(async move {
                let pool = PgPool::connect(&self.url)
                    .await
                    .map_err(|e| MigrationError::Connection(e.to_string()))?;
                let migrator = self.migrator().await?;

                let before: HashSet<i64> = applied_versions(&pool).await;
                if before.is_empty() {
                    return Err(MigrationError::NoMigrationToRevert);
                }
                let target = *before.iter().max().unwrap();

                migrator
                    .undo(&pool, 1)
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
                let pool = PgPool::connect(&self.url)
                    .await
                    .map_err(|e| MigrationError::Connection(e.to_string()))?;
                let migrator = self.migrator().await?;
                let applied = applied_versions(&pool).await;

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

    async fn applied_versions(pool: &PgPool) -> HashSet<i64> {
        sqlx::query_scalar::<_, i64>("SELECT version FROM _sqlx_migrations")
            .fetch_all(pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .collect()
    }
}

// ── sqlite ────────────────────────────────────────────────────────────────────

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
mod sq {
    use super::*;
    use sqlx::SqlitePool;

    impl MigrationRunner for SqlxMigrationRunner {
        fn run(&self) -> BoxFuture<'_, Result<Vec<Migration>, MigrationError>> {
            Box::pin(async move {
                let pool = SqlitePool::connect(&self.url)
                    .await
                    .map_err(|e| MigrationError::Connection(e.to_string()))?;
                let migrator = self.migrator().await?;
                let before = applied_versions(&pool).await;
                migrator
                    .run(&pool)
                    .await
                    .map_err(|e| MigrationError::Apply { version: 0, reason: e.to_string() })?;
                let after = applied_versions(&pool).await;
                let newly_applied = migrator
                    .migrations
                    .iter()
                    .filter(|m| !m.migration_type.is_down_migration()
                        && !before.contains(&m.version)
                        && after.contains(&m.version))
                    .map(|m| Migration::pending(m.version, m.description.as_ref()))
                    .collect();
                Ok(newly_applied)
            })
        }

        fn revert(&self) -> BoxFuture<'_, Result<Migration, MigrationError>> {
            Box::pin(async move {
                let pool = SqlitePool::connect(&self.url)
                    .await
                    .map_err(|e| MigrationError::Connection(e.to_string()))?;
                let migrator = self.migrator().await?;
                let before = applied_versions(&pool).await;
                if before.is_empty() {
                    return Err(MigrationError::NoMigrationToRevert);
                }
                let target = *before.iter().max().unwrap();
                migrator
                    .undo(&pool, 1)
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
                let pool = SqlitePool::connect(&self.url)
                    .await
                    .map_err(|e| MigrationError::Connection(e.to_string()))?;
                let migrator = self.migrator().await?;
                let applied = applied_versions(&pool).await;
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

    async fn applied_versions(pool: &SqlitePool) -> HashSet<i64> {
        sqlx::query_scalar::<_, i64>("SELECT version FROM _sqlx_migrations")
            .fetch_all(pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .collect()
    }
}
