use std::{collections::HashMap, fs::read_to_string, path::PathBuf};

use cs25_303_core::database::DatabaseConfig;
use cs25_303_core::user::auth::AuthenticationProvidersConfig;
use serde::{Deserialize, Serialize};
use strum::EnumIs;
use tracing::Level;

use crate::logging::LoggingLevels;
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
/// The configuration for the application.
///
/// All fields are optional so we support reading from environment variables or configuration files or a mix of both.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ReadConfigType {
    pub mode: Option<Mode>,
    pub web_server: Option<WebServerConfig>,
    pub database: Option<DatabaseConfig>,
    pub log: Option<LoggingConfig>,
    pub tls: Option<TlsConfig>,
    pub auth: Option<AuthenticationProvidersConfig>,
}

#[derive(Debug, Clone, Default, Serialize)]

pub struct FullConfig {
    pub mode: Mode,

    pub web_server: WebServerConfig,
    pub database: DatabaseConfig,
    pub log: LoggingConfig,
    pub tls: Option<TlsConfig>,
    pub auth: AuthenticationProvidersConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebServerConfig {
    pub bind_address: String,
    pub port: u16,
    pub open_api_routes: bool,
}
impl Default for WebServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 8080,
            #[cfg(debug_assertions)]
            open_api_routes: true,
            #[cfg(not(debug_assertions))]
            open_api_routes: false,
        }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TlsConfig {
    pub private_key: PathBuf,
    pub certificate_chain: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub logging_directory: PathBuf,
    pub tracing: Option<TracingConfig>,
    pub stdout_log_levels: Option<LoggingLevels>,
    pub file_log_levels: Option<LoggingLevels>,
}
impl Default for LoggingConfig {
    fn default() -> Self {
        let logging_dir = PathBuf::from("logs");
        Self {
            logging_directory: logging_dir,
            tracing: None,
            stdout_log_levels: None,
            file_log_levels: None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TracingType {
    GRPC,
    /// Not Implemented Yet
    HTTP,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TracingConfig {
    pub tracing_enabled: bool,
    pub tracing_type: TracingType,
    /// Endpoint for the tracing collector.
    ///
    ///
    pub endpoint: String,
    /// Tracing Config Resource Values.
    ///
    /// ```toml
    /// "service.name" = "cs-25-303"
    /// "service.version" = "0.1.0"
    /// "service.environment" = "development"
    /// ```
    pub trace_config: HashMap<String, String>,

    pub log_levels: Option<LoggingLevels>,
}
impl TracingConfig {
    pub fn default_log_levels() -> HashMap<String, Level> {
        let mut log_levels = HashMap::new();
        log_levels.insert("cs-25-303".to_string(), Level::INFO);
        log_levels
    }
}
impl Default for TracingConfig {
    fn default() -> Self {
        let mut trace_config = HashMap::new();
        trace_config.insert("service.name".to_string(), "cs-25-303".to_string());
        trace_config.insert(
            "service.version".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );
        trace_config.insert("service.environment".to_string(), "development".to_string());
        Self {
            tracing_enabled: false,
            tracing_type: TracingType::GRPC,
            endpoint: "127.0.0.1:5959".to_owned(),
            trace_config,
            log_levels: None,
        }
    }
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
    let (web_server, auth, log, database, mode) = env_or_file_or_default!(
        config_from_file,
        environment,
        web_server,
        auth,
        log,
        database,
        mode
    );

    let tls = environment.tls.or(config_from_file.tls.take());

    Ok(FullConfig {
        mode,
        web_server,
        database,
        tls,
        log,
        auth,
    })
}
