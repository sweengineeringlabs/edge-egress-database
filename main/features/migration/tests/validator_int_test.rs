//! Tests for the `Validator` trait contract.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use swe_edge_egress_database_migration::Validator;

/// A minimal struct implementing [`Validator`] for test purposes.
struct AlwaysValid;

impl Validator for AlwaysValid {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// A minimal struct implementing [`Validator`] that always fails for test purposes.
struct AlwaysInvalid;

impl Validator for AlwaysInvalid {
    fn validate(&self) -> Result<(), String> {
        Err("invalid".into())
    }
}

#[test]
fn test_validator_trait_valid_impl_returns_ok() {
    let v = AlwaysValid;
    assert!(v.validate().is_ok());
}

#[test]
fn test_validator_trait_invalid_impl_returns_err() {
    let v = AlwaysInvalid;
    let err = v.validate().expect_err("must return Err for invalid");
    assert_eq!(err, "invalid");
}
