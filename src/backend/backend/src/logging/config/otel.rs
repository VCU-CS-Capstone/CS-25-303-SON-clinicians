use ahash::{HashMap, HashMapExt};
use opentelemetry::{KeyValue, StringValue};
use opentelemetry_sdk::Resource;
use serde::{Deserialize, Serialize};

use super::{AppLoggerType, ChildLoggingLevels, LoggingLevels};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelResourceMap(pub HashMap<String, String>);
impl Default for OtelResourceMap {
    fn default() -> Self {
        let mut trace_config = HashMap::new();
        trace_config.insert(
            "service.name".to_string(),
            env!("CARGO_PKG_NAME").to_string(),
        );
        trace_config.insert(
            "service.version".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );
        trace_config.insert("service.environment".to_string(), "development".to_string());
        Self(trace_config)
    }
}
impl From<OtelResourceMap> for opentelemetry_sdk::Resource {
    fn from(mut value: OtelResourceMap) -> Self {
        if !value.0.contains_key("service.name") {
            value.0.insert(
                "service.name".to_string(),
                env!("CARGO_PKG_NAME").to_string(),
            );
        }
        let resources: Vec<KeyValue> = value
            .0
            .into_iter()
            .map(|(k, v)| KeyValue::new(k, Into::<StringValue>::into(v)))
            .collect();
        Resource::builder().with_attributes(resources).build()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub protocol: TracingProtocol,
    /// Endpoint for the tracing collector.
    pub endpoint: String,
    /// Tracing Config Resource Values.
    ///
    /// ```toml
    /// "service.name" = "cs-25-303"
    /// "service.version" = "0.1.0"
    /// "service.environment" = "development"
    /// ```
    pub config: OtelResourceMap,
}
impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            protocol: TracingProtocol::GRPC,
            endpoint: "http://localhost:4317".to_owned(),
            config: OtelResourceMap::default(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OtelConfig {
    /// Rather or this logger is enabled
    pub enabled: bool,
    /// Protocol to use for tracing.
    pub protocol: TracingProtocol,
    /// Endpoint for the tracing collector.
    pub endpoint: String,
    /// Tracing Config Resource Values.
    ///
    /// ```toml
    /// "service.name" = "cs-25-303"
    /// "service.version" = "0.1.0"
    /// "service.environment" = "development"
    /// ```
    pub config: OtelResourceMap,
    /// Is Tracing spans enabled
    pub traces: bool,
    /// Is Logging enabled
    pub logs: bool,
    /// Child Logger Levels
    pub levels: ChildLoggingLevels,
}
impl AppLoggerType for OtelConfig {
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
impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            protocol: TracingProtocol::GRPC,
            endpoint: "http://localhost:4317".to_owned(),
            config: OtelResourceMap::default(),
            traces: true,
            logs: true,
            levels: ChildLoggingLevels::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TracingProtocol {
    GRPC,
    /// Not Implemented Yet
    HttpBinary,
    HttpJson,
}
impl From<TracingProtocol> for opentelemetry_otlp::Protocol {
    fn from(value: TracingProtocol) -> Self {
        match value {
            TracingProtocol::GRPC => opentelemetry_otlp::Protocol::Grpc,
            TracingProtocol::HttpBinary => opentelemetry_otlp::Protocol::HttpBinary,
            TracingProtocol::HttpJson => opentelemetry_otlp::Protocol::HttpJson,
        }
    }
}
