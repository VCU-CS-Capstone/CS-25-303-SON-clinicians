use axum::body::Body;
use axum::response::Response;
use http::StatusCode;
use http::header::ToStrError;
use thiserror::Error;

use crate::utils::IntoErrorResponse;

#[derive(Debug, Error)]
pub enum BadRequestErrors {
    #[error("Could not Decode Base64: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error("Invalid UTF-8: {0}")]
    InvalidUTF8(#[from] std::string::FromUtf8Error),
    #[error("Invalid Header: {0}")]
    InvalidHeader(#[from] ToStrError),
    #[error("Invalid Header Time: {0}")]
    InvalidHeaderTime(#[from] chrono::ParseError),

    #[error("{0}")]
    Other(String),
    #[error(transparent)]
    Axum(#[from] axum::Error),
    #[error("Missing Header: {0}")]
    MissingHeader(&'static str),
    #[error("Invalid Json Request: {0}")]
    InvalidJson(#[from] serde_json::Error),
}
impl IntoErrorResponse for BadRequestErrors {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from(self.to_string()))
            .unwrap()
    }
}
