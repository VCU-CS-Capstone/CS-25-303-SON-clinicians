pub mod database;
pub mod red_cap;
pub mod user;
pub mod utils;
pub mod env_utils {

    use ahash::{HashMap, HashMapExt};
    pub type EnvMap = HashMap<String, String>;
    #[cfg(test)]
    pub fn read_env_file_in_core(name: &str) -> anyhow::Result<EnvMap> {
        read_env_file_in_dir(name, env!("CARGO_MANIFEST_DIR"))
    }
    pub fn read_env_file_in_dir(name: &str, dir: &str) -> anyhow::Result<EnvMap> {
        let path = std::path::Path::new(dir).join(name);
        if !path.exists() {
            eprintln!("File does not exist: {:?}", path);
            return Ok(EnvMap::default());
        }
        let file_contents = std::fs::read_to_string(path)?;
        Ok(parse_env_file(file_contents))
    }
    pub fn parse_env_file(file_contents: String) -> EnvMap {
        let lines = file_contents.lines();
        let mut env_map = HashMap::with_capacity(lines.size_hint().0);
        for line in file_contents.lines() {
            if should_skip_line(line) {
                continue;
            }
            if !line.contains("=") {
                eprintln!("Invalid line in env file: {}", line);
                continue;
            }
            let mut parts = line.splitn(2, '=');

            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                env_map.insert(key.to_string(), value.to_string());
            }
        }
        env_map
    }
    #[inline]
    fn should_skip_line(line: &str) -> bool {
        let trimmed_line = line.trim();
        trimmed_line.starts_with('#') || trimmed_line.starts_with("//") || trimmed_line.is_empty()
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_parse_env_file() {
            let file_contents = r#"
# This is a comment
KEY1=value1
KEY2=value2
"#;
            let env_map = parse_env_file(file_contents.to_string());
            assert_eq!(env_map.len(), 2);
            assert_eq!(env_map.get("KEY1"), Some(&"value1".to_string()));
            assert_eq!(env_map.get("KEY2"), Some(&"value2".to_string()));
        }
        #[test]
        fn test_sample_test_env_file() {
            let contents = super::read_env_file_in_core("test.env.sample").unwrap();
            for (key, value) in contents.iter() {
                println!("{}: {}", key, value);
            }
        }
    }
}
#[cfg(test)]
pub mod test_utils {
    use std::sync::Once;

    use tracing::{error, info, level_filters::LevelFilter};
    use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt, Layer};

    pub fn init_logger() {
        static ONCE: Once = std::sync::Once::new();
        ONCE.call_once(|| {
            let stdout_log = tracing_subscriber::fmt::layer().pretty().without_time();
            tracing_subscriber::registry()
                .with(
                    stdout_log.with_filter(
                        filter::Targets::new()
                            .with_target("cs25_303_core", LevelFilter::DEBUG)
                            .with_target("sqlx", LevelFilter::INFO),
                    ),
                )
                .init();
        });
        info!("Logger initialized");
        error!("This is an error message");
    }
}
