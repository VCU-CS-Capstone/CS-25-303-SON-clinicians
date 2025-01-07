use crate::app::SiteState;
use axum::body::Body;
use axum::body::HttpBody;
use derive_more::derive::From;
use futures::future::BoxFuture;
use futures::future::LocalBoxFuture;
use http::{Request, Response};
use opentelemetry::metrics::{Counter, UpDownCounter};
use opentelemetry::{global, KeyValue};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::Layer;
use tower_service::Service;

use tracing::{debug, trace};
#[derive(Debug, Clone, From)]
pub struct AppMetricLayer(pub SiteState);

impl<S> Layer<S> for AppMetricLayer {
    type Service = AppMetricsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        let meter = global::meter("axum-request-metrics");

        AppMetricsMiddleware {
            inner,
            site: self.0.clone(),
            metrics: AppMetrics {
                total_requests: meter.u64_counter("total_requests").build(),
                successful_requests: meter.u64_counter("successful_requests").build(),
                failed_requests: meter.u64_counter("failed_requests").build(),
                request_size: meter.i64_up_down_counter("request_size").build(),
                response_size: meter.i64_up_down_counter("response_size").build(),
                request_duration: meter
                    .f64_up_down_counter("request_duration")
                    .with_unit("ms")
                    .build(),
            },
        }
    }
}
#[derive(Debug, Clone)]
pub struct AppMetrics {
    pub total_requests: Counter<u64>,
    pub successful_requests: Counter<u64>,
    pub failed_requests: Counter<u64>,
    pub request_size: UpDownCounter<i64>,
    pub response_size: UpDownCounter<i64>,
    pub request_duration: UpDownCounter<f64>,
}

/// Middleware that handles the authentication of the user
#[derive(Debug, Clone)]
pub struct AppMetricsMiddleware<S> {
    inner: S,
    site: SiteState,
    metrics: AppMetrics,
}

impl<S> Service<Request<Body>> for AppMetricsMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Send + Sync + Clone + 'static,
    S::Future: Send + 'static,
{
    type Response = axum::response::Response<Body>;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, S::Error>>;
    // Async Stuff we can ignore
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        self.metrics.total_requests.add(1, &[]);
        self.metrics
            .request_size
            .add(req.body().size_hint().lower() as i64, &[]);
        // Continue the request
        let mut inner = self.inner.clone();
        let metrics = self.metrics.clone();

        Box::pin(async move {
            let start = std::time::Instant::now();
            let result = inner.call(req).await;
            let duration = start.elapsed().as_millis() as f64;
            metrics.request_duration.add(duration, &[]);
            match &result {
                Ok(response) => {
                    metrics.successful_requests.add(1, &[]);
                    metrics
                        .response_size
                        .add(response.body().size_hint().lower() as i64, &[]);
                }
                Err(_) => {
                    metrics.failed_requests.add(1, &[]);
                }
            }
            result
        })
    }
}
