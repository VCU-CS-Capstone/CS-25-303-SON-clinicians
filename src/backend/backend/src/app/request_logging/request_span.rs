use std::{borrow::Cow, net::SocketAddr};

use axum::extract::{ConnectInfo, FromRef, FromRequestParts, MatchedPath};
use derive_more::derive::From;
use http::{
    HeaderMap, HeaderName, Request,
    header::{REFERER, USER_AGENT},
    request::Parts,
};
use opentelemetry::{global, propagation::Extractor, trace::TraceContextExt};
use tracing::{Level, event, field::Empty, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::{
    app::{SiteState, error::MissingInternelExtension},
    utils::{HeaderMapExt, ip_addr},
};

use super::RequestId;
#[derive(Debug, Clone, From)]
pub struct ErrorReason {
    pub reason: Cow<'static, str>,
}
impl From<String> for ErrorReason {
    fn from(reason: String) -> Self {
        Self {
            reason: Cow::Owned(reason),
        }
    }
}
impl From<&'static str> for ErrorReason {
    fn from(reason: &'static str) -> Self {
        Self {
            reason: Cow::Borrowed(reason),
        }
    }
}
#[derive(Debug, Clone, From)]
pub struct RequestSpan(pub tracing::Span);
impl<S> FromRequestParts<S> for RequestSpan
where
    SiteState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = MissingInternelExtension;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let extension = parts.extensions.get::<RequestSpan>();
        extension
            .cloned()
            .ok_or(MissingInternelExtension("Request Span"))
    }
}
pub fn extract_header_as_str(headers: &HeaderMap, header: HeaderName) -> Option<String> {
    headers
        .get(&header)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            if v.is_empty() {
                event!(Level::WARN, ?header, "Empty header Value",);
                None
            } else {
                Some(v.to_owned())
            }
        })
}

pub struct HeaderMapCarrier<'a>(pub &'a HeaderMap);

impl Extractor for HeaderMapCarrier<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(HeaderName::as_str).collect()
    }
}

pub fn make_span<B>(request: &Request<B>, request_id: RequestId) -> tracing::Span {
    let user_agent = request.headers().get_string_ignore_empty(&USER_AGENT);

    let span = info_span!(target: "cs25_303_backend::requests","HTTP request",
        http.path = Empty,
        http.method = ?request.method(),
        http.version = ?request.version(),
        http.user_agent = user_agent,
        http.client_ip = Empty,
        otel.kind = ?opentelemetry::trace::SpanKind::Server,
        http.status_code = Empty,
        http.referer = Empty,
        http.raw_path = ?request.uri().path(),
        otel.status_code = Empty,
        otel.name = "HTTP request",
        trace_id = Empty,
        exception.message = Empty,
        request_id = display(request_id),
    );

    let context = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderMapCarrier(request.headers()))
    });

    if context.has_active_span() {
        span.record(
            "trace_id",
            context.span().span_context().trace_id().to_string(),
        );
        span.set_parent(context);
    }

    span
}

pub fn on_request<B>(request: &Request<B>, span: &tracing::Span, state: &SiteState) {
    let path = request
        .extensions()
        .get::<MatchedPath>()
        .map_or(request.uri().path(), |p| p.as_str());
    let method = request.method().as_str();
    let client_ip = ip_addr::extract_ip_as_string(request, state);

    span.record("http.path", path);
    span.record("otel.name", format!("{method} {path}"));
    span.record("http.client_ip", client_ip);

    let referer = extract_header_as_str(request.headers(), REFERER);
    if let Some(referer) = referer {
        span.record("http.referer", &referer);
    }
}
pub fn on_response<B>(
    response: &axum::http::Response<B>,
    _latency: std::time::Duration,
    span: &tracing::Span,
) {
    if response.status().is_client_error() || response.status().is_server_error() {
        let reason = response.extensions().get::<ErrorReason>();
        if let Some(reason) = reason {
            span.record("exception.message", reason.reason.as_ref());
        } else {
            span.record("exception.message", "Unknown error");
        }
    }

    span.record("http.status_code", response.status().as_u16());
    span.record("otel.status_code", "OK");
}
pub fn on_failure<C>(
    _failure_classification: C,
    _latency: std::time::Duration,
    span: &tracing::Span,
) {
    span.record("otel.status_code", "ERROR");
}
