use std::{error::Error, fmt::Display};

use axum::{body::Body, response::IntoResponse};
use http::header::CONTENT_TYPE;
pub mod bad_request;
pub trait IntoErrorResponse: Error + Send + Sync {
    /// Converts the error into a response
    ///
    /// It must be of type of Box<Self> to allow for dynamic dispatch
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response;
    #[inline(always)]
    fn json_error_response(self: Box<Self>) -> Option<axum::response::Response> {
        None
    }
    #[inline(always)]
    fn supports_json_error_response(&self) -> bool {
        false
    }
}
// TODO: Improve the error message to be easier to read.
fn internal_error_message(err: impl Error, source: &'static str) -> Body {
    format!(
        "Internal Service Error. Please Contact the System Admin.
        Error: {}
        Source: {source},
        ",
        err
    )
    .into()
}
fn internal_service_error_response(
    err: impl Error,
    source: &'static str,
) -> axum::response::Response {
    let body = internal_error_message(err, source);
    axum::response::Response::builder()
        .status(http::StatusCode::INTERNAL_SERVER_ERROR)
        .body(body)
        .unwrap()
}
#[derive(serde::Serialize)]
struct JSONErrorResponse {
    error: String,
    source: &'static str,
}
fn json_error_response(err: impl Error, source: &'static str) -> axum::response::Response {
    match serde_json::to_string(&JSONErrorResponse {
        error: err.to_string(),
        source,
    }) {
        Ok(ok) => axum::response::Response::builder()
            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
            .header(CONTENT_TYPE, "application/json")
            .body(ok.into())
            .unwrap(),
        Err(err) => {
            let body = internal_error_message(err, source);
            axum::response::Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .header(CONTENT_TYPE, "text/plain")
                .body(body)
                .unwrap()
        }
    }
}

macro_rules! basic_internal_error {
    (
        $(
            $error:path => $source:literal
        ),*
    ) => {
        $(
            impl IntoErrorResponse for $error {
                fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
                    internal_service_error_response(self, $source)
                }
                fn json_error_response(self: Box<Self>) -> Option<axum::response::Response> {
                    Some(json_error_response(self, $source))
                }
                fn supports_json_error_response(&self) -> bool {
                    true
                }
            }
        )*
    };

}
basic_internal_error!(
    std::io::Error => "IO",
    sqlx::Error => "Database",
    cs25_303_core::database::DBError => "Database",
    // Do not use this when handing user input. An error message saying request error should be returned.
    serde_json::Error => "JSON",
    http::Error => "HTTP",
    argon2::Error => "Argon2",
    argon2::password_hash::Error => "Argon2"
);
#[derive(Debug)]
pub struct InternalError(pub Box<dyn IntoErrorResponse>);
impl Display for InternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for InternalError {}

impl IntoResponse for InternalError {
    fn into_response(self) -> axum::response::Response {
        self.0.into_response_boxed()
    }
}

impl<T: IntoErrorResponse + 'static> From<T> for InternalError {
    fn from(err: T) -> Self {
        InternalError(Box::new(err))
    }
}
