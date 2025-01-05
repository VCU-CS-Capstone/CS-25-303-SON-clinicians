use axum::{body::Body, response::Response};
use http::StatusCode;
use strum::EnumIs;
use thiserror::Error;
use tracing::{error, instrument, warn};

use crate::app::error::IntoErrorResponse;
#[derive(Debug, Error)]
pub enum InvalidAuthorizationHeader {
    #[error("Invalid Authorization Scheme")]
    InvalidScheme,
    #[error("Invalid Authorization Value")]
    InvalidValue,
    #[error("Invalid Authorization Format. Expected: (Schema Type) (Value)")]
    InvalidFormat,
    #[error("Not a valid UTF-8 string {0}")]
    ToStrError(#[from] http::header::ToStrError),
}
impl IntoErrorResponse for InvalidAuthorizationHeader {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(self.to_string()))
            .unwrap()
    }
}

#[derive(Debug, EnumIs)]
pub enum AuthorizationHeader {
    /// Http Bearer Token
    Bearer {
        token: String,
    },
    // This is not an official schema, however, it can be used to pass a session token without using Cookies
    Session {
        session: String,
    },
}
impl TryFrom<String> for AuthorizationHeader {
    type Error = InvalidAuthorizationHeader;
    #[instrument(skip(value), name = "AuthorizationHeader::try_from")]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split(' ').collect();

        if parts.len() != 2 {
            return Err(InvalidAuthorizationHeader::InvalidFormat);
        }
        let scheme = parts[0];
        let value = parts[1];
        match scheme {
            "Bearer" => Ok(AuthorizationHeader::Bearer {
                token: value.to_owned(),
            }),

            "Session" => Ok(AuthorizationHeader::Session {
                session: value.to_owned(),
            }),
            schema => {
                warn!(?schema, "Unknown Authorization Scheme");
                Err(InvalidAuthorizationHeader::InvalidScheme)
            }
        }
    }
}
