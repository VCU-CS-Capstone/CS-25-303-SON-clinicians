mod otel;
use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use ahash::{HashMap, HashMapExt};
use derive_more::derive::From;
pub use otel::*;
use serde::{Deserialize, Serialize};
use tracing::{Level, level_filters::LevelFilter};
use tracing_appender::rolling::Rotation;
use tracing_subscriber::{
    filter::Targets,
    fmt::{
        format::{self, Format},
        time::SystemTime,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub loggers: HashMap<String, AppLogger>,
    pub metrics: Option<MetricsConfig>,
    /// The default logging levels.
    pub levels: LoggingLevels,
}
impl LoggingConfig {
    pub fn add_logger(&mut self, name: impl Into<String>, logger: impl Into<AppLogger>) {
        self.loggers.insert(name.into(), logger.into());
    }
}
impl Default for LoggingConfig {
    fn default() -> Self {
        let mut config = Self {
            loggers: HashMap::with_capacity(3),
            metrics: Some(MetricsConfig::default()),
            levels: LoggingLevels::actual_default(),
        };
        config.add_logger("app", OtelConfig::default());
        config.add_logger("console", ConsoleLogger::default());
        config.add_logger("file", RollingFileLogger::default());
        config
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, From)]
#[serde(tag = "type", content = "config")]
pub enum AppLogger {
    Otel(OtelConfig),
    Console(ConsoleLogger),
    RollingFile(RollingFileLogger),
}
pub trait AppLoggerType {
    fn enabled(&self) -> bool;
    fn inherit_levels_from_parent(&self) -> bool;
    fn get_levels_mut(&mut self) -> &mut LoggingLevels;
}
impl AppLoggerType for AppLogger {
    fn enabled(&self) -> bool {
        match self {
            AppLogger::Otel(config) => config.enabled(),
            AppLogger::Console(config) => config.enabled(),
            AppLogger::RollingFile(config) => config.enabled(),
        }
    }
    fn inherit_levels_from_parent(&self) -> bool {
        match self {
            AppLogger::Otel(config) => config.inherit_levels_from_parent(),
            AppLogger::Console(config) => config.inherit_levels_from_parent(),
            AppLogger::RollingFile(config) => config.inherit_levels_from_parent(),
        }
    }
    fn get_levels_mut(&mut self) -> &mut LoggingLevels {
        match self {
            AppLogger::Otel(config) => config.get_levels_mut(),
            AppLogger::Console(config) => config.get_levels_mut(),
            AppLogger::RollingFile(config) => config.get_levels_mut(),
        }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct ChildLoggingLevels {
    /// Rather or not to inherit from the global levels.
    pub inherit: Option<bool>,
    #[serde(flatten)]
    pub levels: LoggingLevels,
}
impl Deref for ChildLoggingLevels {
    type Target = LoggingLevels;

    fn deref(&self) -> &Self::Target {
        &self.levels
    }
}
impl DerefMut for ChildLoggingLevels {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.levels
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct LoggingLevels {
    pub default: Option<LevelSerde>,
    pub others: HashMap<String, LevelSerde>,
}
impl LoggingLevels {
    pub fn with_default(mut self, level: LevelSerde) -> Self {
        self.default = Some(level);
        self
    }
    pub fn with_target(mut self, target: impl Into<String>, level: impl Into<LevelSerde>) -> Self {
        self.others.insert(target.into(), level.into());
        self
    }
    pub fn add_target(&mut self, target: impl Into<String>, level: impl Into<LevelSerde>) {
        self.others.insert(target.into(), level.into());
    }
}
impl From<LoggingLevels> for Targets {
    fn from(targets: LoggingLevels) -> Self {
        let mut builder = tracing_subscriber::filter::Targets::new();
        if let Some(default) = targets.default {
            builder = builder.with_default(default);
        }
        for (name, level) in targets.others {
            builder = builder.with_target(name, level);
        }
        builder
    }
}

impl Default for LoggingLevels {
    fn default() -> Self {
        Self {
            default: Some(LevelSerde::Info),
            others: Default::default(),
        }
    }
}
impl LoggingLevels {
    pub fn actual_default() -> Self {
        Self::default()
            .with_default(LevelSerde::Info)
            .with_target("cs_25_303_backend", LevelSerde::Debug)
            .with_target("cs_25_303_core", LevelSerde::Debug)
            .with_target("h2", LevelSerde::Warn)
            .with_target("tower", LevelSerde::Warn)
            .with_target("tonic", LevelSerde::Warn)
            .with_target("hyper_util", LevelSerde::Warn)
    }
}
impl LoggingLevels {
    /// Inherit the levels from another logging levels.
    ///
    /// This will check if Self contains a key from other if not it will insert it.
    pub fn inherit_from(&mut self, other: &LoggingLevels) {
        for (k, v) in other.others.iter() {
            if !self.others.contains_key(k) {
                self.others.insert(k.clone(), v.clone());
            }
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum LevelSerde {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    Off,
}
impl From<LevelSerde> for LevelFilter {
    fn from(level: LevelSerde) -> Self {
        match level {
            LevelSerde::Error => LevelFilter::ERROR,
            LevelSerde::Warn => LevelFilter::WARN,
            LevelSerde::Info => LevelFilter::INFO,
            LevelSerde::Debug => LevelFilter::DEBUG,
            LevelSerde::Trace => LevelFilter::TRACE,
            LevelSerde::Off => LevelFilter::OFF,
        }
    }
}
impl From<LevelFilter> for LevelSerde {
    fn from(level: LevelFilter) -> Self {
        match level {
            LevelFilter::ERROR => LevelSerde::Error,
            LevelFilter::WARN => LevelSerde::Warn,
            LevelFilter::INFO => LevelSerde::Info,
            LevelFilter::DEBUG => LevelSerde::Debug,
            LevelFilter::TRACE => LevelSerde::Trace,
            LevelFilter::OFF => LevelSerde::Off,
        }
    }
}
impl From<Level> for LevelSerde {
    fn from(level: Level) -> Self {
        match level {
            Level::ERROR => LevelSerde::Error,
            Level::WARN => LevelSerde::Warn,
            Level::INFO => LevelSerde::Info,
            Level::DEBUG => LevelSerde::Debug,
            Level::TRACE => LevelSerde::Trace,
        }
    }
}
impl TryFrom<LevelSerde> for Level {
    type Error = ();

    fn try_from(value: LevelSerde) -> Result<Self, Self::Error> {
        match value {
            LevelSerde::Error => Ok(Level::ERROR),
            LevelSerde::Warn => Ok(Level::WARN),
            LevelSerde::Info => Ok(Level::INFO),
            LevelSerde::Debug => Ok(Level::DEBUG),
            LevelSerde::Trace => Ok(Level::TRACE),
            _ => Err(()),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StandardLoggerFmtRules {
    pub include_time: bool,
    pub pretty: bool,
    pub include_level: bool,
    pub include_line_numbers: bool,
    pub include_file: bool,
    pub include_target: bool,
    pub ansi_color: bool,
    pub include_thread_ids: bool,
    pub include_thread_names: bool,
}
impl Default for StandardLoggerFmtRules {
    fn default() -> Self {
        Self {
            pretty: false,
            include_time: true,
            include_level: true,
            include_line_numbers: false,
            include_file: false,
            include_target: true,
            ansi_color: true,
            include_thread_ids: false,
            include_thread_names: false,
        }
    }
}
impl StandardLoggerFmtRules {
    pub fn layer_pretty<S>(
        &self,
    ) -> tracing_subscriber::fmt::Layer<S, format::Pretty, format::Format<format::Pretty, SystemTime>>
    {
        self.layer().pretty()
    }
    pub fn layer_compact<S>(
        &self,
    ) -> tracing_subscriber::fmt::Layer<S, format::DefaultFields, Format<format::Compact, SystemTime>>
    {
        self.layer().compact()
    }
    pub fn layer<S>(
        &self,
    ) -> tracing_subscriber::fmt::Layer<S, format::DefaultFields, Format<format::Full, SystemTime>>
    {
        tracing_subscriber::fmt::layer::<S>()
            .with_ansi(self.ansi_color)
            .with_target(self.include_target)
            .with_line_number(self.include_line_numbers)
            .with_file(self.include_file)
            .with_level(self.include_level)
            .with_thread_ids(self.include_thread_ids)
            .with_thread_names(self.include_thread_names)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ConsoleLogger {
    pub enabled: bool,
    pub pretty: bool,
    #[serde(flatten)]
    pub rules: StandardLoggerFmtRules,
    pub levels: ChildLoggingLevels,
}
impl AppLoggerType for ConsoleLogger {
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn inherit_levels_from_parent(&self) -> bool {
        self.levels.inherit.unwrap_or(true)
    }
    fn get_levels_mut(&mut self) -> &mut LoggingLevels {
        &mut self.levels.levels
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollingFileLogger {
    pub enabled: bool,
    pub path: PathBuf,
    pub file_prefix: String,
    pub levels: ChildLoggingLevels,

    pub interval: RollingInterval,
    #[serde(flatten)]
    pub rules: StandardLoggerFmtRules,
}
impl AppLoggerType for RollingFileLogger {
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn inherit_levels_from_parent(&self) -> bool {
        self.levels.inherit.unwrap_or(true)
    }
    fn get_levels_mut(&mut self) -> &mut LoggingLevels {
        &mut self.levels.levels
    }
}
impl Default for RollingFileLogger {
    fn default() -> Self {
        Self {
            enabled: true,
            path: PathBuf::from("logs/"),
            file_prefix: "cs-25.log".to_string(),
            levels: ChildLoggingLevels::default(),
            interval: RollingInterval::Daily,
            rules: StandardLoggerFmtRules {
                ansi_color: false,
                ..Default::default()
            },
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollingInterval {
    Minutely,
    Hourly,
    Daily,
    Never,
}

impl From<RollingInterval> for Rotation {
    fn from(value: RollingInterval) -> Self {
        match value {
            RollingInterval::Minutely => Rotation::MINUTELY,
            RollingInterval::Hourly => Rotation::HOURLY,
            RollingInterval::Daily => Rotation::DAILY,
            RollingInterval::Never => Rotation::NEVER,
        }
    }
}
