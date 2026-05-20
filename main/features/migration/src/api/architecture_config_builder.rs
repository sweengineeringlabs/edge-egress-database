//! Builder types for migration architecture configuration.
//!
//! Impl blocks live in the `saf` layer. Struct shapes are declared here so
//! types are anchored in the interface layer per SEA rule 160.

/// Builder for migration architecture configuration.
///
/// Construct via [`crate::saf::builder`]. Finalize with [`ArchitectureConfigBuilder::build`].
#[allow(dead_code)]
pub struct ArchitectureConfigBuilder {
    _private: (),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architecture_config_builder_constructs() {
        let _b = ArchitectureConfigBuilder { _private: () };
    }
}
