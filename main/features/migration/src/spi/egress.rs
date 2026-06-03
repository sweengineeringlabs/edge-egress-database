//! Extension re-exports for downstream [`MigrationRunner`] consumers.
//!
//! This module is the SEA SPI anchor. Its presence signals that callers
//! may substitute a custom [`MigrationRunner`] implementation.

/// Extension point marker for downstream migration runner substitution.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SEA spi/ anchor — constructed only by the inline anchor test"
    )
)]
pub(crate) struct Egress;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_egress_is_constructible_as_spi_anchor() {
        let _e = Egress;
    }
}
