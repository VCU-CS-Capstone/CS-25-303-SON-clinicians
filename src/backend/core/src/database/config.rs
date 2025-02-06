use serde::{Deserialize, Serialize};
use sqlx::postgres::PgConnectOptions;

use super::DBError;

/// The configuration for the database.
///
/// Currently only supports PostgreSQL.
#[derive(Debug, Clone, Deserialize, Serialize, clap::Args)]
pub struct DatabaseConfig {
    #[clap(long = "database-user", default_value = "postgres")]
    pub user: String,
    #[clap(long = "database-password", default_value = "password")]
    pub password: String,
    #[clap(long = "database-name", default_value = "cs25_303")]
    pub database: String,
    // The host can be in the format host:port or just host.
    #[clap(long = "database-host", default_value = "localhost")]
    pub host: String,
    // The port is optional. If not specified the default port is used. or will be extracted from the host.
    #[clap(long = "database-port", default_value = "5432")]
    pub port: u16,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            user: "postgres".to_string(),
            password: "password".to_string(),
            database: "cs_25_303".to_string(),
            host: "localhost".to_string(),
            port: 5432,
        }
    }
}
impl TryFrom<DatabaseConfig> for PgConnectOptions {
    type Error = DBError;
    fn try_from(settings: DatabaseConfig) -> Result<PgConnectOptions, Self::Error> {
        let options = PgConnectOptions::new()
            .username(&settings.user)
            .password(&settings.password)
            .host(&settings.host)
            .port(settings.port)
            .database(&settings.database);

        Ok(options)
    }
}
