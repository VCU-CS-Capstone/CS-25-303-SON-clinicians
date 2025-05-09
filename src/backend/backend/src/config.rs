use std::{fs::read_to_string, path::PathBuf};

use cs25_303_core::database::DatabaseConfig;
use cs25_303_core::user::auth::AuthenticationProvidersConfig;
use serde::{Deserialize, Serialize};
use strum::EnumIs;
use utoipa::ToSchema;
pub mod robots;
use crate::logging::config::LoggingConfig;
pub const CONFIG_PREFIX: &str = "CS-25-303";
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, EnumIs)]
pub enum Mode {
    Debug,
    Release,
}
impl Default for Mode {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        return Mode::Debug;
        #[cfg(not(debug_assertions))]
        return Mode::Release;
    }
}
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct EnabledFeatures {
    /// Rather or not to enabled the scalar openapi page
    pub scalar: bool,
    /// Rather or not to enable the openapi routes
    pub open_api_routes: bool,
    /// Rather or not to enable participant data updates
    pub update_participant_data: bool,
    /// Rather or not to enable syncing data from redcap to this system
    pub red_cap_read_syncing: bool,
    /// Rather to sync data written to this system to red cap
    pub red_cap_write_syncing: bool,
}
impl Default for EnabledFeatures {
    fn default() -> Self {
        Self {
            scalar: true,
            open_api_routes: true,
            update_participant_data: true,
            red_cap_read_syncing: false,
            red_cap_write_syncing: false,
        }
    }
}
/// The configuration for the application.
///
/// All fields are optional so we support reading from environment variables or configuration files or a mix of both.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ReadConfigType {
    pub mode: Option<Mode>,
    pub enabled_features: Option<EnabledFeatures>,
    pub web_server: Option<WebServerConfig>,
    pub database: Option<DatabaseConfig>,
    pub log: Option<LoggingConfig>,
    pub tls: Option<TlsConfig>,
    pub auth: Option<AuthenticationProvidersConfig>,
    pub robots: Option<robots::RobotsConfig>,
}

#[derive(Debug, Clone, Default, Serialize)]

pub struct FullConfig {
    pub mode: Mode,
    pub enabled_features: EnabledFeatures,
    pub web_server: WebServerConfig,
    pub database: DatabaseConfig,
    pub log: LoggingConfig,
    pub tls: Option<TlsConfig>,
    pub auth: AuthenticationProvidersConfig,
    pub robots: robots::RobotsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebServerConfig {
    pub bind_address: String,
    pub port: u16,
}
impl Default for WebServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 8080,
        }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TlsConfig {
    pub private_key: PathBuf,
    pub certificate_chain: PathBuf,
}

macro_rules! env_or_file_or_default {
    (
        $config:ident,
        $env:ident,
        $key:ident
    ) => {
        $config.$key.or($env.$key).unwrap_or_default()
    };
    ( $config:ident, $env:ident, $($key:ident),* ) => {
        (
            $(
                env_or_file_or_default!($config, $env, $key),
            )*
        )
    }
}
/// Load the configuration from the environment or a configuration file.
///
/// path: may not exist if it doesn't it will use the environment variables.
pub fn load_config(path: Option<PathBuf>) -> anyhow::Result<FullConfig> {
    let environment: ReadConfigType = serde_env::from_env_with_prefix(CONFIG_PREFIX)?;
    let mut config_from_file =
        if let Some(path) = path.filter(|path| path.exists() && path.is_file()) {
            let contents = read_to_string(path)?;
            toml::from_str(&contents)?
        } else {
            ReadConfigType::default()
        };
    // Merge the environment variables with the configuration file. If neither exists the default values are used.
    // Environment variables take precedence.
    let (web_server, auth, log, database, mode, enabled_features, robots) = env_or_file_or_default!(
        config_from_file,
        environment,
        web_server,
        auth,
        log,
        database,
        mode,
        enabled_features,
        robots
    );

    let tls = environment.tls.or(config_from_file.tls.take());

    Ok(FullConfig {
        mode,
        web_server,
        database,
        tls,
        log,
        auth,
        enabled_features,
        robots,
    })
}
