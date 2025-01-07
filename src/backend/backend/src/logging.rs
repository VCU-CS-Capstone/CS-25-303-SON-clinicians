use std::vec;

use crate::config::{LoggingConfig, MetricsConfig, Mode, TracingConfig};
use ahash::{HashMap, HashMapExt};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::StringValue;
use opentelemetry::{global, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, MetricExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::logs::{Logger, LoggerProvider};
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::{Tracer, TracerProvider};
use opentelemetry_sdk::{propagation::TraceContextPropagator, Resource};
use serde::{Deserialize, Serialize};
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::{Layer, Registry};
fn tracer(mut config: TracingConfig) -> anyhow::Result<Tracer> {
    println!("Loading Tracing {config:#?}");

    if !config.config.contains_key("service.name") {
        config
            .config
            .insert("service.name".to_owned(), "cs25_303".to_owned());
    }
    let resources: Vec<KeyValue> = config
        .config
        .into_iter()
        .map(|(k, v)| KeyValue::new(k, Into::<StringValue>::into(v)))
        .collect();
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_protocol(config.protocol.into())
        .with_endpoint(&config.endpoint);
    let provider = TracerProvider::builder()
        .with_resource(Resource::new(resources))
        .with_batch_exporter(exporter.build()?, opentelemetry_sdk::runtime::Tokio)
        .build();
    Ok(provider.tracer("tracing-otel-subscriber"))
}

fn logger(
    mut config: TracingConfig,
) -> anyhow::Result<OpenTelemetryTracingBridge<LoggerProvider, Logger>> {
    println!("Loading Tracing {config:#?}");

    if !config.config.contains_key("service.name") {
        config
            .config
            .insert("service.name".to_owned(), "cs25_303".to_owned());
    }
    let resources: Vec<KeyValue> = config
        .config
        .into_iter()
        .map(|(k, v)| KeyValue::new(k, Into::<StringValue>::into(v)))
        .collect();
    let exporter = LogExporter::builder()
        .with_tonic()
        .with_protocol(config.protocol.into())
        .with_endpoint(&config.endpoint);
    let provider = LoggerProvider::builder()
        .with_resource(Resource::new(resources))
        .with_batch_exporter(exporter.build()?, opentelemetry_sdk::runtime::Tokio)
        .build();
    Ok(OpenTelemetryTracingBridge::new(&provider))
}

fn metrics(mut config: MetricsConfig) -> anyhow::Result<SdkMeterProvider> {
    println!("Loading Tracing {config:#?}");

    if !config.config.contains_key("service.name") {
        config
            .config
            .insert("service.name".to_owned(), "cs25_303".to_owned());
    }
    let resources: Vec<KeyValue> = config
        .config
        .into_iter()
        .map(|(k, v)| KeyValue::new(k, Into::<StringValue>::into(v)))
        .collect();
    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_protocol(config.protocol.into())
        .with_endpoint(&config.endpoint)
        .build()?;
    let reader = PeriodicReader::builder(exporter, opentelemetry_sdk::runtime::Tokio).build();

    Ok(SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(Resource::new(resources))
        .build())
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
    let mut layers = vec![fmt_layer.boxed(), file.boxed()];

    if let Some(otel) = config.otel_logger {
        if otel.enabled {
            let otel_filter: Targets = otel
                .log_levels
                .clone()
                .unwrap_or_else(|| default_otel_levels(mode))
                .into();
            let logger = logger(otel)?;
            layers.push(logger.with_filter(otel_filter).boxed());
        }
    }
    if let Some(tracing) = config.tracing {
        let otel_filter: Targets = tracing
            .log_levels
            .clone()
            .unwrap_or_else(|| default_otel_levels(mode))
            .into();
        let tracer = tracer(tracing)?;
        let otel_layer = tracing_subscriber::Layer::with_filter(
            tracing_opentelemetry::layer().with_tracer(tracer).boxed(),
            otel_filter,
        );
        layers.push(otel_layer.boxed());
    }
    let subscriber = Registry::default().with(layers);
    subscriber.init();
    info!("Logging initialized");
    if let Some(metrics_config) = config.otel_metrics {
        if metrics_config.enabled {
            let provider = metrics(metrics_config)?;
            global::set_meter_provider(provider);
        }
    }
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
            others: HashMap::default(),
        }
    }
}
pub fn default_otel_levels(mode: Mode) -> LoggingLevels {
    let mut others = HashMap::new();
    others.insert("cs25_303_backend".to_string(), LevelSerde::Trace);
    others.insert("cs25_303_core".to_string(), LevelSerde::Trace);
    others.insert("h2".to_string(), LevelSerde::Warn);
    others.insert("tower".to_string(), LevelSerde::Warn);
    others.insert("tonic".to_string(), LevelSerde::Warn);
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
            others.insert("tonic".to_string(), LevelSerde::Warn);
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
