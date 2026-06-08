//! [`DatabaseConfig`] — backend-owned `[database]` TOML contract.

use swe_edge_configbuilder::{ConfigError, FeatureMetadata, OptionalSection};

use crate::api::types::driver_kind::DriverKind;

/// Canonical configuration for the `[database]` TOML section.
///
/// Per ADR-006, this struct is owned by `swe-edge-egress-database-migration`:
/// the crate that opens the datasource also defines the section name, field
/// shape, and validation rules. Consumers opt in by adding this crate to
/// `Cargo.toml` and a `[database]` section to `application.toml` — they never
/// redefine the struct.
///
/// The feature is enabled by the **presence** of the `[database]` section.
/// Because this struct uses `#[serde(deny_unknown_fields)]`, do **not** write
/// `enabled = true` (the loader rejects it as an unknown field); `enabled =
/// false` is the explicit opt-out and is interpreted before deserialization.
///
/// # Examples
///
/// SQLite:
/// ```toml
/// [database]
/// driver = "sqlite"
/// url    = "sqlite:///var/lib/obsrv/data.db"
/// migrations_dir = "./migrations"
/// ```
///
/// PostgreSQL with pool tuning:
/// ```toml
/// [database]
/// driver               = "postgres"
/// url                  = "postgres://obsrv:secret@db.internal:5432/obsrv"
/// max_connections      = 16
/// acquire_timeout_secs = 10
/// idle_timeout_secs    = 300
/// migrations_dir       = "./migrations"
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatabaseConfig {
    /// Which backend to open.
    pub driver: DriverKind,

    /// Connection URL.
    ///
    /// - `sqlite`: `sqlite://<path>` / `sqlite:///<abs-path>` / `sqlite::memory:`
    /// - `postgres`: `postgres://user:pass@host:port/db`
    pub url: String,

    /// Maximum number of connections the pool may open. Must be `>= 1`.
    #[serde(default = "DatabaseConfig::default_max_connections")]
    pub max_connections: u32,

    /// How long to wait for a free connection before returning a timeout error.
    #[serde(default = "DatabaseConfig::default_acquire_timeout_secs")]
    pub acquire_timeout_secs: u64,

    /// Close idle connections after this many seconds. `None` keeps them open.
    #[serde(default)]
    pub idle_timeout_secs: Option<u64>,

    /// Directory of migration `.sql` files (sqlx naming: `<version>_<desc>.sql`).
    ///
    /// Required by `connect_and_migrate`; optional for `connect` (pool only).
    #[serde(default)]
    pub migrations_dir: Option<String>,
}

impl DatabaseConfig {
    /// Default pool size when `max_connections` is omitted.
    fn default_max_connections() -> u32 {
        5
    }

    /// Default acquire timeout (seconds) when omitted.
    fn default_acquire_timeout_secs() -> u64 {
        30
    }
}

impl OptionalSection for DatabaseConfig {
    // @allow: no_stub_fn_bodies — returns the canonical section key, not a stub
    fn section_name() -> &'static str {
        "database"
    }

    fn validate_enabled(&self) -> Result<(), ConfigError> {
        if self.url.trim().is_empty() {
            return Err(ConfigError::validation(
                Self::section_name(),
                "`url` must be non-empty",
            ));
        }
        if self.max_connections == 0 {
            return Err(ConfigError::validation(
                Self::section_name(),
                "`max_connections` must be >= 1",
            ));
        }
        let scheme = self.driver.url_scheme();
        if !self.url.starts_with(scheme) {
            return Err(ConfigError::validation(
                Self::section_name(),
                format!(
                    "driver = \"{:?}\" requires a `url` starting with \"{}\"; got \"{}\"",
                    self.driver, scheme, self.url
                ),
            ));
        }
        Ok(())
    }

    fn metadata() -> FeatureMetadata {
        FeatureMetadata {
            description: "relational database datasource + schema migrations",
            owner: "platform-team",
            deprecated_since: None,
        }
    }
}
