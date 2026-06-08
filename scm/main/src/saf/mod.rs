//! SAF layer — public factory surface for the datasource + migration runner.

mod migration_svc;

pub use crate::api::error::MigrationError;
pub use crate::api::migration::{Migration, MigrationStatus};
pub use crate::api::types::ApplicationConfigBuilder;
pub use crate::api::types::DatabaseConfig;
pub use crate::api::types::DriverKind;
pub use crate::api::types::MigrationSvc;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
pub use crate::spi::deadpool::DbPool;
