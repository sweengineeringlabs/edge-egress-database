/// Builder for configuration.
#[derive(Debug, Default)]
pub struct ApplicationConfigBuilder {
    _private: (),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_config_builder_constructs() {
        let _b = ApplicationConfigBuilder::default();
    }
}
