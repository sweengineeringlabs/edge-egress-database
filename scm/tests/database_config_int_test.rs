//! Tests for `DatabaseConfig` — the backend-owned `[database]` OptionalSection.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use swe_edge_configbuilder::OptionalSection;
use swe_edge_egress_database_migration::{DatabaseConfig, DriverKind};

/// @covers: OptionalSection::section_name — canonical `[database]` key.
#[test]
fn test_database_config_section_name_is_database() {
    assert_eq!(DatabaseConfig::section_name(), "database");
}

/// @covers: serde deserialize — full section with explicit pool tuning.
#[test]
fn test_database_config_deserializes_postgres_with_pool_tuning() {
    let cfg: DatabaseConfig = toml::from_str(
        r#"
        driver               = "postgres"
        url                  = "postgres://u:p@db:5432/obsrv"
        max_connections      = 16
        acquire_timeout_secs = 10
        idle_timeout_secs    = 300
        migrations_dir       = "./migrations"
        "#,
    )
    .expect("valid [database] section must deserialize");

    assert_eq!(cfg.driver, DriverKind::Postgres);
    assert_eq!(cfg.max_connections, 16);
    assert_eq!(cfg.acquire_timeout_secs, 10);
    assert_eq!(cfg.idle_timeout_secs, Some(300));
    assert_eq!(cfg.migrations_dir.as_deref(), Some("./migrations"));
}

/// @covers: serde defaults — omitted tuning fields use documented defaults.
#[test]
fn test_database_config_applies_defaults_for_omitted_fields() {
    let cfg: DatabaseConfig = toml::from_str(
        r#"
        driver = "sqlite"
        url    = "sqlite:///data.db"
        "#,
    )
    .expect("minimal [database] section must deserialize");

    assert_eq!(cfg.driver, DriverKind::Sqlite);
    assert_eq!(cfg.max_connections, 5, "default max_connections");
    assert_eq!(cfg.acquire_timeout_secs, 30, "default acquire timeout");
    assert_eq!(cfg.idle_timeout_secs, None);
    assert_eq!(cfg.migrations_dir, None);
}

/// @covers: deny_unknown_fields — an unknown key (incl. `enabled = true`) is rejected.
#[test]
fn test_database_config_rejects_unknown_field() {
    let err = toml::from_str::<DatabaseConfig>(
        r#"
        driver  = "sqlite"
        url     = "sqlite:///data.db"
        enabled = true
        "#,
    )
    .expect_err("unknown field must be rejected by deny_unknown_fields");
    assert!(
        err.to_string().contains("enabled") || err.to_string().contains("unknown"),
        "error should mention the offending field: {err}",
    );
}

/// @covers: validate_enabled — a well-formed config passes.
#[test]
fn test_validate_enabled_accepts_consistent_driver_and_url() {
    let cfg = DatabaseConfig {
        driver: DriverKind::Postgres,
        url: "postgres://u:p@db/obsrv".into(),
        max_connections: 8,
        acquire_timeout_secs: 30,
        idle_timeout_secs: None,
        migrations_dir: None,
    };
    assert!(cfg.validate_enabled().is_ok());
}

/// @covers: validate_enabled — empty url is rejected.
#[test]
fn test_validate_enabled_rejects_empty_url() {
    let cfg = DatabaseConfig {
        driver: DriverKind::Sqlite,
        url: "   ".into(),
        max_connections: 5,
        acquire_timeout_secs: 30,
        idle_timeout_secs: None,
        migrations_dir: None,
    };
    assert!(
        cfg.validate_enabled().is_err(),
        "empty url must be rejected"
    );
}

/// @covers: validate_enabled — zero max_connections is rejected.
#[test]
fn test_validate_enabled_rejects_zero_max_connections() {
    let cfg = DatabaseConfig {
        driver: DriverKind::Sqlite,
        url: "sqlite:///data.db".into(),
        max_connections: 0,
        acquire_timeout_secs: 30,
        idle_timeout_secs: None,
        migrations_dir: None,
    };
    assert!(
        cfg.validate_enabled().is_err(),
        "max_connections = 0 must be rejected"
    );
}

/// @covers: validate_enabled — driver/url scheme mismatch is rejected.
#[test]
fn test_validate_enabled_rejects_driver_url_scheme_mismatch() {
    let cfg = DatabaseConfig {
        driver: DriverKind::Postgres,
        url: "sqlite:///data.db".into(),
        max_connections: 5,
        acquire_timeout_secs: 30,
        idle_timeout_secs: None,
        migrations_dir: None,
    };
    let err = cfg
        .validate_enabled()
        .expect_err("postgres driver with sqlite url must be rejected");
    assert!(
        err.to_string().contains("postgres") || err.to_string().contains("url"),
        "error should explain the scheme mismatch: {err}",
    );
}

/// @covers: DriverKind::url_scheme — neutral scheme strings.
#[test]
fn test_driver_kind_url_scheme_strings() {
    assert_eq!(DriverKind::Sqlite.url_scheme(), "sqlite:");
    assert_eq!(DriverKind::Postgres.url_scheme(), "postgres");
}
