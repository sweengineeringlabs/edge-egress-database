//! [`DriverKind`] — selects which database backend a [`DatabaseConfig`] activates.
//!
//! [`DatabaseConfig`]: crate::DatabaseConfig

/// Which database backend [`DatabaseConfig`] targets.
///
/// Deserialized from the `driver` key of the `[database]` TOML section using
/// snake_case spellings: `"sqlite"` and `"postgres"`.
///
/// This type is technology-neutral (it names no driver library); the concrete
/// deadpool pool that backs each variant lives in `spi/` per ADR-008.
///
/// [`DatabaseConfig`]: crate::DatabaseConfig
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriverKind {
    /// SQLite file (or `:memory:`) backend. Requires the `sqlite` feature.
    Sqlite,
    /// PostgreSQL server backend. Requires the `postgres` feature.
    Postgres,
}

impl DriverKind {
    /// The URL scheme prefix this driver expects (`"sqlite:"` / `"postgres"`).
    pub fn url_scheme(&self) -> &'static str {
        match self {
            DriverKind::Sqlite => "sqlite:",
            DriverKind::Postgres => "postgres",
        }
    }
}
