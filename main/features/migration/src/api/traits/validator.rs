//! Validator trait for the migration crate.

/// Validates configuration or input before use.
pub trait Validator {
    /// Validate this value.
    ///
    /// Returns `Ok(())` if the value is valid, or a human-readable error message.
    fn validate(&self) -> Result<(), String>;
}
