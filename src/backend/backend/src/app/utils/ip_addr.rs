use axum::extract::{ConnectInfo, FromRef, FromRequestParts};
use http::{HeaderName, request::Parts};
use std::net::SocketAddr;

use crate::utils::HeaderMapExt;
#[allow(clippy::declare_interior_mutable_const)]
pub const X_FORWARDED_FOR_HEADER: HeaderName = HeaderName::from_static("x-forwarded-for");
/// Tries to rely on the `x-forwarded-for` header to get the client's IP address.
/// This is useful when the server is behind a reverse proxy.
///
/// If the header is not present it will try to get the IP address from the connection info.
/// If the connection info is not present it will return `None`.
#[derive(Debug)]
pub struct ConnectionIpAddr(pub Option<String>);
impl<S> FromRequestParts<S> for ConnectionIpAddr
where
    S: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let client_ip = parts
            .headers
            .get_string(X_FORWARDED_FOR_HEADER)
            .or_else(|| {
                parts
                    .extensions
                    .get::<ConnectInfo<SocketAddr>>()
                    .map(|ConnectInfo(c)| c.to_string())
            });
        Ok(ConnectionIpAddr(client_ip))
    }
}
