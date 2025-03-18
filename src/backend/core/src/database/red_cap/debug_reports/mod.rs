use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;
pub mod goals;
pub mod medications;
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct DebugParticipantSummary {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub program: String,
    pub location: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sqlx(default)]
    pub location_name: Option<String>,
}
