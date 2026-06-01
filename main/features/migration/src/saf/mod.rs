//! SAF layer — public factory surface for migration runners.

mod migration_svc;

pub use crate::api::error::MigrationError;
pub use crate::api::migration::{Migration, MigrationStatus};
pub use crate::api::types::ApplicationConfigBuilder;
pub use crate::api::types::MigrationSvc;
