use serde::Serialize;
use utoipa::ToSchema;

use crate::app::error::InternalError;
use axum::{body::Body, response::Response};
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
        .body(Body::empty())?)
}
#[derive(Serialize, ToSchema)]
pub struct ConflictResponse {
    pub fields: Vec<String>,
}
impl From<&str> for ConflictResponse {
    fn from(field: &str) -> Self {
        ConflictResponse {
            fields: vec![field.to_string()],
        }
    }
}
impl From<Vec<String>> for ConflictResponse {
    fn from(fields: Vec<String>) -> Self {
        ConflictResponse { fields }
    }
}
impl From<String> for ConflictResponse {
    fn from(field: String) -> Self {
        ConflictResponse {
            fields: vec![field],
        }
    }
}

impl ConflictResponse {
    pub fn response(self) -> Result<Response, InternalError> {
        let body = serde_json::to_string(&self)?;
        Ok(http::Response::builder()
            .status(http::StatusCode::CONFLICT)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(body.into())?)
    }
}
