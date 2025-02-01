use std::sync::Once;

use super::db::DBTestingConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoreTestingConfig {
    pub database: DBTestingConfig,
    pub red_cap_token: Option<String>,
}
impl CoreTestingConfig {
    pub fn init_logger(&self) {
        // TODO: Add Config to logger
        use tracing::error;
        use tracing::info;
        use tracing::level_filters::LevelFilter;
        use tracing_subscriber::filter;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;
        use tracing_subscriber::Layer;
        static ONCE: Once = std::sync::Once::new();
        ONCE.call_once(|| {
            let stdout_log: tracing_subscriber::fmt::Layer<
                tracing_subscriber::Registry,
                tracing_subscriber::fmt::format::Pretty,
                tracing_subscriber::fmt::format::Format<
                    tracing_subscriber::fmt::format::Pretty,
                    (),
                >,
            > = tracing_subscriber::fmt::layer().pretty().without_time();
            tracing_subscriber::registry()
                .with(
                    stdout_log.with_filter(
                        filter::Targets::new()
                            .with_target("cs25_303_core", LevelFilter::TRACE)
                            .with_target("sqlx", LevelFilter::INFO),
                    ),
                )
                .init();
        });
        info!("Logger initialized");
        error!("This is an error message");
    }
}
#[cfg(test)]
pub mod testing {
    #![allow(dead_code)]
    use std::{
        env,
        io::{self, Write},
        path::PathBuf,
        sync::OnceLock,
    };

    use crate::utils::testing::find_file_with_name_check_parents;

    use super::CoreTestingConfig;

    static CONFIG_CELL: OnceLock<Option<CoreTestingConfig>> = OnceLock::new();
    static DB_CONNECTION: OnceLock<Option<sqlx::PgPool>> = OnceLock::new();

    fn load_config() -> Result<Option<CoreTestingConfig>, anyhow::Error> {
        let file_path = match std::env::var_os("CS25_CORE_TEST_CONFIG").map(PathBuf::from) {
            Some(path) => path,
            None => {
                let current_dir = env::current_dir()?;
                let Some(path) = find_file_with_name_check_parents(
                    current_dir,
                    "cs-25-303-core.testing.toml",
                    3,
                ) else {
                    return Ok(None);
                };
                found_config_at_path(&path)?;
                path
            }
        };
        let content = std::fs::read_to_string(&file_path)?;
        let config: CoreTestingConfig = toml::from_str(&content)?;

        Ok(Some(config))
    }
    pub fn get_testing_config() -> Option<CoreTestingConfig> {
        let config = CONFIG_CELL.get_or_init(|| {
            let config = load_config().expect("Error loading config");
            config
        });
        config.clone()
    }
    pub fn found_config_at_path(path: &PathBuf) -> Result<(), anyhow::Error> {
        let mut stout_locked = io::stdout().lock();
        stout_locked.write_all(format!("Found config at path: {}\n", path.display()).as_bytes())?;
        Ok(())
    }
    pub fn no_testing_config() -> Result<(), anyhow::Error> {
        let mut stout_locked = io::stderr().lock();
        stout_locked.write_all(b"No testing config found\n")?;
        Ok(())
    }
    pub fn no_db_connection() -> Result<(), anyhow::Error> {
        let mut stout_locked = io::stderr().lock();
        stout_locked.write_all(b"Database not configured in config file")?;
        Ok(())
    }
}
