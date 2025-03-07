use super::header::{AuthorizationHeader, InvalidAuthorizationHeader};
use crate::app::SiteState;
use crate::app::authentication::AuthenticationRaw;
use crate::app::error::InternalError;
use crate::app::request_logging::{RequestId, RequestSpan};
use crate::utils::HeaderValueExt;
use axum::body::Body;
use axum_extra::extract::CookieJar;
use derive_more::derive::From;
use http::header::AUTHORIZATION;
use http::request::Parts;
use http::{Request, Response};
use pin_project::pin_project;
use std::task::ready;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::Layer;
use tower_service::Service;
use tracing::field::Empty;
use tracing::{Span, info, info_span, trace};
#[derive(Debug, Clone, From)]
pub struct AuthenticationLayer(pub SiteState);

impl<S> Layer<S> for AuthenticationLayer {
    type Service = AuthenticationMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthenticationMiddleware {
            inner,
            site: self.0.clone(),
        }
    }
}
/// Middleware that handles the authentication of the user
#[derive(Debug, Clone)]
pub struct AuthenticationMiddleware<S> {
    inner: S,
    site: SiteState,
}
impl<S> AuthenticationMiddleware<S> {
    pub fn process_from_parts(&self, parts: &mut Parts, span: &Span) -> Result<(), InternalError> {
        let cookie_jar = CookieJar::from_headers(&parts.headers);
        let authorization_header = parts
            .headers
            .get(AUTHORIZATION)
            .map(|header| header.parsed::<AuthorizationHeader, InvalidAuthorizationHeader>())
            .transpose()?;
        if let Some(auth) = authorization_header {
            span.record("auth.method", "Authorization Header");
            trace!("Authorization Header Received");
            parts
                .extensions
                .insert(AuthenticationRaw::new_from_auth_header(auth, &self.site));
        } else if let Some(cookie) = cookie_jar.get("session") {
            span.record("auth.method", "Session Cookie");
            trace!("Session Cookie Received");
            parts
                .extensions
                .insert(AuthenticationRaw::new_from_cookie(cookie, &self.site));
        } else {
            trace!("No Authentication Header or Cookie Found");
        }
        Ok(())
    }
}

impl<S> Service<Request<Body>> for AuthenticationMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Send + Sync + Clone + 'static,
    S::Future: Send + 'static,
    S::Error: std::fmt::Display + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;
    // Async Stuff we can ignore
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let parent_span = req
            .extensions()
            .get::<RequestSpan>()
            .map(|span| {
                println!("Request Span(From Extension) {:?}", span.0);
                span.0.clone()
            })
            .unwrap_or_else(Span::current);

        if req.method() == http::Method::OPTIONS {
            // Options requests are ignored
            trace!("Options Request");
            let inner = parent_span.in_scope(|| self.inner.call(req));
            return ResponseFuture {
                inner: Kind::Ok { future: inner },
            };
        }
        let request_id = req
            .extensions()
            .get::<RequestId>()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "<unknown>".to_string());
        info!("Executing Authentication Middleware");
        let (mut parts, body) = req.into_parts();

        {
            let span = info_span!(
                parent: &parent_span,
                "Authentication Middleware",
                project_module = "Authentication",
                otel.status_code = Empty,
                exception.message = Empty,
                auth.method = Empty,
                trace_id = Empty,
                request_id = request_id,
            );
            let _auth_guard = span.enter();
            if let Err(error) = self.process_from_parts(&mut parts, &span) {
                span.record("exception.message", error.to_string());
                span.record("otel.status_code", "ERROR");
                return ResponseFuture {
                    inner: Kind::InvalidAuthentication {
                        error: error.to_string(),
                    },
                };
            } else {
                span.record("otel.status_code", "OK");
            }
        }

        // Continue the request
        let req = Request::from_parts(parts, body);
        let inner = parent_span.in_scope(|| self.inner.call(req));
        ResponseFuture {
            inner: Kind::Ok { future: inner },
        }
    }
}

/// Async Wrapper for Response
#[pin_project]
pub struct ResponseFuture<F> {
    #[pin]
    inner: Kind<F>,
}

#[pin_project(project = KindProj)]
enum Kind<F> {
    /// Authentication was able to be parsed and will continue
    Ok {
        #[pin]
        future: F,
    },
    /// An unparsable authentication header was passed
    InvalidAuthentication { error: String },
}
impl<F, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<Body>, E>>,
{
    type Output = Result<Response<Body>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.inner.project() {
            KindProj::InvalidAuthentication { error } => {
                let body = Body::from(format!("Invalid Authentication Header: {}", error));
                let response = Response::new(body);
                Poll::Ready(Ok(response))
            }
            KindProj::Ok { future } => {
                let response: Response<Body> = ready!(future.poll(cx))?;

                Poll::Ready(Ok(response))
            }
        }
    }
}
