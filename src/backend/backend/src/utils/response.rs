use serde::Serialize;
use utoipa::ToSchema;
pub mod builder;
use crate::app::error::InternalError;
use axum::response::Response;
pub use builder::ResponseBuilder;

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
