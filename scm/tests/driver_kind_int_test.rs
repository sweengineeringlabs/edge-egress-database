//! Integration tests for [`DriverKind`].

use swe_edge_egress_database_migration::DriverKind;

/// @covers: DriverKind::url_scheme — Sqlite variant returns the correct URL prefix
#[test]
fn test_driver_kind_url_scheme_sqlite_returns_sqlite_colon() {
    assert_eq!(DriverKind::Sqlite.url_scheme(), "sqlite:");
}

/// @covers: DriverKind::url_scheme — Postgres variant returns the correct URL prefix
#[test]
fn test_driver_kind_url_scheme_postgres_returns_postgres() {
    assert_eq!(DriverKind::Postgres.url_scheme(), "postgres");
}

/// @covers: DriverKind — Copy and equality hold across both variants
#[test]
fn test_driver_kind_copy_and_equality() {
    let a = DriverKind::Sqlite;
    let b = a; // Copy
    assert_eq!(a, b);
    assert_ne!(DriverKind::Sqlite, DriverKind::Postgres);
}

/// @covers: DriverKind — deserialization from snake_case TOML strings
#[test]
fn test_driver_kind_deserializes_from_snake_case() {
    let sqlite: DriverKind =
        toml::from_str::<toml::Value>("value = \"sqlite\"")
            .unwrap()
            .get("value")
            .unwrap()
            .clone()
            .try_into()
            .unwrap();
    assert_eq!(sqlite, DriverKind::Sqlite);

    let postgres: DriverKind =
        toml::from_str::<toml::Value>("value = \"postgres\"")
            .unwrap()
            .get("value")
            .unwrap()
            .clone()
            .try_into()
            .unwrap();
    assert_eq!(postgres, DriverKind::Postgres);
}
