use std::collections::HashMap;

use crate::config::{LoggingConfig, Mode, TracingConfig};
use opentelemetry::trace::TracerProvider;
use opentelemetry::StringValue;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::{new_exporter, WithExportConfig};
use opentelemetry_sdk::trace::{Config as SDKTraceConfig, Tracer};
use opentelemetry_sdk::{propagation::TraceContextPropagator, Resource};
use serde::{Deserialize, Serialize};
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::Layer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn tracer(mut config: TracingConfig) -> anyhow::Result<Tracer> {
    println!("Loading Tracing {config:#?}");

    if !config.trace_config.contains_key("service.name") {
        config
            .trace_config
            .insert("service.name".to_owned(), "cs25_303".to_owned());
    }
    let resources: Vec<KeyValue> = config
        .trace_config
        .into_iter()
        .map(|(k, v)| KeyValue::new(k, Into::<StringValue>::into(v)))
        .collect();
    let trace_config = SDKTraceConfig::default().with_resource(Resource::new(resources));

    let exporter = new_exporter().tonic().with_endpoint(&config.endpoint);
    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(trace_config)
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;
    Ok(provider.tracer("tracing-otel-subscriber"))
}

pub fn init(config: LoggingConfig, mode: Mode) -> anyhow::Result<()> {
    let std_out_filter: Targets = config
        .stdout_log_levels
        .unwrap_or_else(|| default_other_levels(mode))
        .into();
    let file_filter: Targets = config
        .file_log_levels
        .unwrap_or_else(|| default_other_levels(mode))
        .into();

    let fmt_layer = tracing_subscriber::Layer::with_filter(
        tracing_subscriber::fmt::layer().pretty(),
        std_out_filter,
    );
    // Rolling File fmt_layer
    let file = {
        let file_appender =
            tracing_appender::rolling::hourly(config.logging_directory, "cs25_303.log");
        tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_file(true)
            .with_level(true)
            .with_writer(file_appender)
            .with_filter(file_filter)
    };
    global::set_text_map_propagator(TraceContextPropagator::new());

    let registry = tracing_subscriber::registry().with(fmt_layer).with(file);

    if let Some(tracing) = config.tracing {
        let otel_filter: Targets = tracing
            .log_levels
            .clone()
            .unwrap_or_else(|| default_otel_levels(mode))
            .into();
        let tracer = tracer(tracing)?;
        let otel_layer = tracing_subscriber::Layer::with_filter(
            tracing_opentelemetry::layer().with_tracer(tracer),
            otel_filter,
        );
        registry.with(otel_layer).init();
    } else {
        registry.init();
    }

    info!("Logging initialized");
    Ok(())
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingLevels {
    pub default: LevelSerde,
    pub others: HashMap<String, LevelSerde>,
}
impl From<LoggingLevels> for Targets {
    fn from(targets: LoggingLevels) -> Self {
        let mut builder = tracing_subscriber::filter::Targets::new();

        builder = builder.with_default(targets.default);
        for (name, level) in targets.others {
            builder = builder.with_target(name, level);
        }
        builder
    }
}

impl Default for LoggingLevels {
    fn default() -> Self {
        Self {
            default: LevelSerde::Info,
            others: HashMap::new(),
        }
    }
}
pub fn default_otel_levels(mode: Mode) -> LoggingLevels {
    let mut others = HashMap::new();
    others.insert("cs25_303_backend".to_string(), LevelSerde::Trace);
    others.insert("cs25_303_core".to_string(), LevelSerde::Trace);
    others.insert("h2".to_string(), LevelSerde::Warn);
    others.insert("tower".to_string(), LevelSerde::Warn);
    others.insert("hyper_util".to_string(), LevelSerde::Warn);
    let default = match mode {
        Mode::Debug => LevelSerde::Trace,
        Mode::Release => LevelSerde::Info,
    };
    LoggingLevels { default, others }
}

pub fn default_other_levels(mode: Mode) -> LoggingLevels {
    match mode {
        Mode::Debug => {
            let mut others = HashMap::new();
            others.insert("cs25_303_backend".to_string(), LevelSerde::Trace);
            others.insert("cs25_303_core".to_string(), LevelSerde::Trace);
            others.insert("h2".to_string(), LevelSerde::Warn);
            others.insert("tower".to_string(), LevelSerde::Warn);
            others.insert("hyper_util".to_string(), LevelSerde::Warn);
            LoggingLevels {
                default: LevelSerde::Trace,
                others,
            }
        }
        Mode::Release => LoggingLevels {
            default: LevelSerde::Info,
            others: HashMap::new(),
        },
    }
}
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum LevelSerde {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
impl From<LevelSerde> for LevelFilter {
    fn from(level: LevelSerde) -> Self {
        match level {
            LevelSerde::Error => LevelFilter::ERROR,
            LevelSerde::Warn => LevelFilter::WARN,
            LevelSerde::Info => LevelFilter::INFO,
            LevelSerde::Debug => LevelFilter::DEBUG,
            LevelSerde::Trace => LevelFilter::TRACE,
        }
    }
}
