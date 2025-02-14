use std::sync::Once;

use crate::database::DBError;

use super::db::DBTestingConfig;
use serde::{Deserialize, Serialize};
use tracing::error;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoreTestingConfig {
    pub database: Option<DBTestingConfig>,
    pub red_cap_token: Option<String>,
}
impl CoreTestingConfig {
    pub async fn connect_to_db(&self) -> Result<sqlx::PgPool, DBError> {
        let db = self.database.as_ref().ok_or_else(|| {
            DBError::Other("No [database] Config Set in `cs-25-303-core.testing.toml`")
        })?;
        db.connect().await
    }
    pub fn init_logger(&self) {
        static ONCE: Once = std::sync::Once::new();

        ONCE.call_once(|| {
            let stdout_log = tracing_subscriber::fmt::layer().pretty().without_time();
            let sqlx_logger = {
                let file_appender =
                    tracing_appender::rolling::hourly("test/testing-logs", "sqlx.log");
                tracing_subscriber::fmt::layer()
                    .pretty()
                    .without_time()
                    .with_ansi(false)
                    .with_writer(file_appender)
                    .with_filter(filter::Targets::new().with_target("sqlx", LevelFilter::DEBUG))
            };

            tracing_subscriber::registry()
                .with(
                    stdout_log.with_filter(
                        filter::Targets::new()
                            .with_target("pg_extended_sqlx_queries", LevelFilter::TRACE)
                            .with_target("cs25_303_core", LevelFilter::TRACE)
                            .with_default(LevelFilter::INFO)
                            .with_target("sqlx", LevelFilter::WARN),
                    ),
                )
                .with(sqlx_logger)
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
        path::{Path, PathBuf},
        sync::OnceLock,
    };
    use tokio::sync::OnceCell as AsyncOnceCell;

    use crate::utils::testing::find_file_with_name_check_parents;

    use super::CoreTestingConfig;

    static CONFIG_CELL: OnceLock<Option<CoreTestingConfig>> = OnceLock::new();
    static DB_NAME: AsyncOnceCell<String> = AsyncOnceCell::const_new();

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
        let config = CONFIG_CELL.get_or_init(|| load_config().expect("Error loading config"));
        config.clone()
    }
    /// Gets the testing database
    ///
    /// We want to be able to reuse the same database during a test run
    ///
    /// However, we can't hold onto the connection pool in a static variable because at the end of each test the tokio runtime is shutdown
    ///
    /// So we hold the database name and create a new connection pool each time
    pub async fn get_testing_db() -> Option<sqlx::PgPool> {
        let config = get_testing_config()?;
        config.init_logger();
        let Some(db_config) = &config.database else {
            no_db_connection().unwrap();
            return None;
        };
        let db_name = DB_NAME
            .get_or_try_init(|| db_config.create_testing_db())
            .await
            .expect("Error getting db");

        let connection = db_config
            .connect_with_name(db_name)
            .await
            .expect("Error connecting to db");
        Some(connection)
    }
    pub fn found_config_at_path(path: &Path) -> Result<(), anyhow::Error> {
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
