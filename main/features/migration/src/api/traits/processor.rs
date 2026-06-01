//! Primary processing trait for the migration crate.

/// Primary processing trait for this crate.
pub trait Processor: Send + Sync {
    /// Identify this processor in log / trace output.
    fn describe(&self) -> &'static str;
}
