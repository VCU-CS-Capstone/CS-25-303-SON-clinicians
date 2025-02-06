use cs25_303_core::database::DatabaseConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DataToolConfig {
    pub red_cap_token: Option<String>,
    pub database: DatabaseConfig,
}

pub fn load_config(
    file_path: impl AsRef<std::path::Path>,
) -> Result<DataToolConfig, anyhow::Error> {
    if !file_path.as_ref().exists() {
        return Ok(DataToolConfig::default());
    }
    let content = std::fs::read_to_string(file_path)?;
    let config: DataToolConfig = toml::from_str(&content)?;
    Ok(config)
}
#[cfg(test)]
pub mod testing {
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct DataToolTestingConfig {
        pub database: Option<DBTestingConfig>,
    }

    use std::{
        env,
        io::{self, Write},
        path::{Path, PathBuf},
    };

    use cs25_303_core::utils::testing::{db::DBTestingConfig, find_file_with_name_check_parents};
    use serde::{Deserialize, Serialize};

    pub fn get_testing_config() -> Result<Option<DataToolTestingConfig>, anyhow::Error> {
        let file_path = match std::env::var_os("CS25_DT_TEST_CONFIG").map(PathBuf::from) {
            Some(path) => path,
            None => {
                let current_dir = env::current_dir()?;
                let Some(path) =
                    find_file_with_name_check_parents(current_dir, "data-tools.testing.toml", 3)
                else {
                    return Ok(None);
                };
                found_config_at_path(&path)?;
                path
            }
        };
        let content = std::fs::read_to_string(&file_path)?;
        let config: DataToolTestingConfig = toml::from_str(&content)?;
        Ok(Some(config))
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
