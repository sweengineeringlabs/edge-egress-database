//! Gateway layer — inbound and outbound integration boundaries.

pub(crate) mod egress;
pub(crate) mod ingress;

pub use crate::api::traits::MigrationRunner;
pub use crate::api::traits::Processor;
pub use crate::api::traits::Validator;
pub use crate::saf::*;
