use config::{
    AppLogger, AppLoggerType, ConsoleLogger, LoggingConfig, LoggingLevels, MetricsConfig,
    OtelConfig, RollingFileLogger,
};
use opentelemetry::{global, trace::TracerProvider};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, MetricExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    logs::SdkLoggerProvider,
    metrics::{PeriodicReader, SdkMeterProvider},
    propagation::TraceContextPropagator,
    trace::SdkTracerProvider,
};
use tracing::debug;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::{
    Layer, Registry, filter::Targets, layer::SubscriberExt, util::SubscriberInitExt,
};

pub mod config;
struct TracerResult {
    levels: LoggingLevels,
    logging: Option<SdkLoggerProvider>,
    tracing: Option<SdkTracerProvider>,
}
fn tracer(config: OtelConfig) -> anyhow::Result<Option<TracerResult>> {
    if !config.enabled {
        return Ok(None);
    }
    let resources: Resource = config.config.into();

    let tracer = if config.traces {
        let exporter = SpanExporter::builder()
            .with_tonic()
            .with_protocol(config.protocol.into())
            .with_endpoint(&config.endpoint);
        let provider = SdkTracerProvider::builder()
            .with_resource(resources.clone())
            .with_batch_exporter(exporter.build()?)
            .build();
        Some(provider)
    } else {
        None
    };
    let logger = if config.logs {
        let exporter = LogExporter::builder()
            .with_tonic()
            .with_protocol(config.protocol.into())
            .with_endpoint(&config.endpoint);
        let provider = SdkLoggerProvider::builder()
            .with_resource(resources.clone())
            .with_batch_exporter(exporter.build()?)
            .build();
        Some(provider)
    } else {
        None
    };

    Ok(Some(TracerResult {
        levels: config.levels.levels,
        logging: logger,
        tracing: tracer,
    }))
}

fn metrics(config: MetricsConfig) -> anyhow::Result<SdkMeterProvider> {
    println!("Loading Tracing {config:#?}");

    let resources: Resource = config.config.into();

    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_protocol(config.protocol.into())
        .with_endpoint(&config.endpoint)
        .build()?;
    let reader = PeriodicReader::builder(exporter).build();

    Ok(SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resources)
        .build())
}

pub fn init(config: LoggingConfig) -> anyhow::Result<LoggingState> {
    let mut layers: Vec<Box<dyn Layer<Registry> + Send + Sync>> =
        Vec::with_capacity(config.loggers.len());
    let mut state = LoggingState {
        items: Vec::with_capacity(config.loggers.len()),
        ..Default::default()
    };
    let LoggingConfig {
        loggers,
        metrics: metrics_config,
        levels: parent_levels,
    } = config;

    for (name, logger) in loggers
        .into_iter()
        .filter(|(_, logger)| logger.enabled())
        .map(|(k, mut logger)| {
            if logger.inherit_levels_from_parent() {
                println!("{k} is Inheriting levels from parent");
                logger.get_levels_mut().inherit_from(&parent_levels);
            }
            (k, logger)
        })
    {
        match logger {
            AppLogger::Otel(config) => {
                let Some(TracerResult {
                    levels,
                    logging,
                    tracing,
                }) = tracer(config)?
                else {
                    continue;
                };
                println!("Otel Levels: {levels:?}");
                state.set_global_text_propagator();
                let logging_levels: Targets = levels.into();
                if let Some(tracer_provider) = tracing {
                    let tracer = tracer_provider.tracer(name);
                    state.items.push(LoggingStateItem::Tracer(tracer_provider));
                    let otel_layer = tracing_subscriber::Layer::with_filter(
                        tracing_opentelemetry::layer().with_tracer(tracer).boxed(),
                        logging_levels.clone(),
                    );
                    layers.push(otel_layer.boxed());
                }
                if let Some(logging_provider) = logging {
                    let tracing_bridge = OpenTelemetryTracingBridge::new(&logging_provider);
                    state.items.push(LoggingStateItem::Logger(logging_provider));

                    let otel_layer =
                        tracing_subscriber::Layer::with_filter(tracing_bridge, logging_levels);

                    layers.push(otel_layer.boxed());
                }
            }
            AppLogger::Console(config) => {
                let ConsoleLogger {
                    pretty,
                    levels,
                    rules,
                    ..
                } = config;
                let logging_levels: Targets = levels.levels.into();
                if pretty {
                    let fmt_layer = rules.layer_pretty().with_filter(logging_levels);
                    layers.push(fmt_layer.boxed());
                } else {
                    let fmt_layer = rules.layer().with_filter(logging_levels);

                    layers.push(fmt_layer.boxed());
                }
            }
            AppLogger::RollingFile(config) => {
                let RollingFileLogger {
                    levels,
                    rules,
                    path,
                    file_prefix,
                    interval,
                    ..
                } = config;
                let logging_levels: Targets = levels.levels.into();

                let file_appender =
                    RollingFileAppender::new(interval.into(), path.clone(), file_prefix.clone());

                let fmt_layer = if rules.pretty {
                    rules
                        .layer()
                        .with_writer(file_appender)
                        .with_filter(logging_levels)
                        .boxed()
                } else {
                    rules
                        .layer_pretty()
                        .with_writer(file_appender)
                        .with_filter(logging_levels)
                        .boxed()
                };

                layers.push(fmt_layer);
            }
        }
    }
    let subscriber = Registry::default().with(layers);
    subscriber.init();
    if let Some(metrics_config) = metrics_config {
        if metrics_config.enabled {
            let provider = metrics(metrics_config)?;
            global::set_meter_provider(provider.clone());
            state.items.push(LoggingStateItem::Meter(provider));
        }
    }
    Ok(state)
}

#[derive(Debug, Default)]
pub struct LoggingState {
    pub items: Vec<LoggingStateItem>,
    has_set_global_text_propagator: bool,
}
impl LoggingState {
    pub fn close(self) -> anyhow::Result<()> {
        // TODO Call shutdown when https://github.com/open-telemetry/opentelemetry-rust/issues/868 is resolved
        for item in self.items {
            debug!("Closing {:?}", item);
        }

        Ok(())
    }

    fn set_global_text_propagator(&mut self) {
        if self.has_set_global_text_propagator {
            return;
        }
        self.has_set_global_text_propagator = true;
        global::set_text_map_propagator(TraceContextPropagator::new());
    }
}
#[derive(Debug)]
pub enum LoggingStateItem {
    Logger(SdkLoggerProvider),
    Tracer(SdkTracerProvider),
    Meter(SdkMeterProvider),
}
