//! `impl Processor for MigrationSvc` — satisfies rule 154.

use crate::api::traits::Processor;
use crate::api::types::MigrationSvc;

impl Processor for MigrationSvc {
    fn describe(&self) -> &'static str {
        const LABEL: &str = "database-migration";
        LABEL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @covers: describe
    #[test]
    fn test_describe_migration_svc_processor_returns_label() {
        let svc = MigrationSvc;
        assert_eq!(
            svc.describe(),
            "database-migration",
            "Processor::describe must return the crate label"
        );
    }
}
