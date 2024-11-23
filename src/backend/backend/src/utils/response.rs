use serde::Serialize;

use crate::app::error::InternalError;
use axum::response::Response;
pub fn ok_json_response<T: Serialize>(data: T) -> Result<Response, InternalError> {
    let body = serde_json::to_string(&data)?;
    Ok(http::Response::builder()
        .status(http::StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(body.into())?)
}

pub fn not_found_response() -> Result<Response, InternalError> {
    Ok(http::Response::builder()
        .status(http::StatusCode::NOT_FOUND)
        .body(String::new().into())?)
}
