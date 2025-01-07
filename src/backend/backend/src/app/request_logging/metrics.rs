use crate::app::SiteState;
use axum::body::Body;
use axum::body::HttpBody;
use axum::extract::MatchedPath;
use derive_more::derive::From;
use futures::future::BoxFuture;
use http::{Request, Response};
use opentelemetry::KeyValue;
use std::task::{Context, Poll};
use tower::Layer;
use tower_service::Service;

#[derive(Debug, Clone, From)]
pub struct AppMetricLayer(pub SiteState);

impl<S> Layer<S> for AppMetricLayer {
    type Service = AppMetricsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AppMetricsMiddleware {
            inner,
            site: self.0.clone(),
        }
    }
}

/// Middleware that handles the authentication of the user
#[derive(Debug, Clone)]
pub struct AppMetricsMiddleware<S> {
    inner: S,
    site: SiteState,
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
        let path = req
            .extensions()
            .get::<MatchedPath>()
            .map_or(req.uri().path(), |p| p.as_str());

        let mut attributes = vec![
            KeyValue::new("http.route", path.to_owned()),
            KeyValue::new("http.request.method", req.method().as_str().to_string()),
        ];
        let site: SiteState = self.site.clone();
        let body_size = req.body().size_hint().lower();

        // Continue the request
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let start = std::time::Instant::now();
            let result = inner.call(req).await;
            let duration = start.elapsed().as_millis() as f64;
            if let Ok(response) = &result {
                attributes.push(KeyValue::new(
                    "http.response.status_code",
                    response.status().as_u16().to_string(),
                ));

                site.metrics
                    .response_size_bytes
                    .record(response.body().size_hint().lower(), &[]);
            }

            site.metrics
                .request_size_bytes
                .record(body_size, &attributes);
            site.metrics
                .request_duration
                .record(duration as f64 / 1000f64, &attributes);
            result
        })
    }
}
