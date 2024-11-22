use super::header::{AuthorizationHeader, InvalidAuthorizationHeader};
use crate::app::authentication::AuthenticationRaw;
use crate::app::SiteState;
use crate::utils::HeaderValueExt;
use axum::body::Body;
use axum_extra::extract::CookieJar;
use derive_more::derive::From;
use http::header::AUTHORIZATION;
use http::{Request, Response};
use http_body_util::Either;
use pin_project::pin_project;
use std::task::ready;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::Layer;
use tower_service::Service;
use tracing::trace;
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
type ServiceBody<T> = Either<T, Body>;
type ServiceResponse<T> = Response<ServiceBody<T>>;
/// Middleware that handles the authentication of the user
#[derive(Debug, Clone)]
pub struct AuthenticationMiddleware<S> {
    inner: S,
    site: SiteState,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for AuthenticationMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ReqBody: Default,
{
    type Response = ServiceResponse<ResBody>;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;
    // Async Stuff we can ignore
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
    /// Parse the request authentication and pass it into an extension
    #[tracing::instrument(
        skip(self, req),
        name = "AuthenticationMiddleware",
        fields(project_module = "Authentication")
    )]
    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        if req.method() == http::Method::OPTIONS {
            // Options requests are ignored
            trace!("Options Request");
            return ResponseFuture {
                inner: Kind::Ok {
                    future: self.inner.call(req),
                },
            };
        }
        let (mut parts, body) = req.into_parts();
        let cookie_jar = CookieJar::from_headers(&parts.headers);
        let authorization_header = parts
            .headers
            .get(AUTHORIZATION)
            .map(|header| header.parsed::<AuthorizationHeader, InvalidAuthorizationHeader>());
        if let Some(auth) = authorization_header {
            match auth {
                Ok(auth) => {
                    parts
                        .extensions
                        .insert(AuthenticationRaw::new_from_auth_header(auth, &self.site));
                }
                Err(err) => {
                    return ResponseFuture {
                        inner: Kind::InvalidAuthentication {
                            error: err.to_string(),
                        },
                    };
                }
            }
        } else if let Some(cookie) = cookie_jar.get("session") {
            parts
                .extensions
                .insert(AuthenticationRaw::new_from_cookie(cookie, &self.site));
        }

        // Continue the request
        ResponseFuture {
            inner: Kind::Ok {
                future: self.inner.call(Request::from_parts(parts, body)),
            },
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
impl<F, B, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<B>, E>>,
{
    type Output = Result<ServiceResponse<B>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project().inner.project() {
            KindProj::InvalidAuthentication { error } => {
                let body = Body::from(format!("Invalid Authentication Header: {}", error));
                let response = Response::new(Either::Right(body));

                Poll::Ready(Ok(response))
            }
            KindProj::Ok { future } => {
                let response: Response<B> = ready!(future.poll(cx))?;
                let response = response.map(Either::Left);
                Poll::Ready(Ok(response))
            }
        }
    }
}
