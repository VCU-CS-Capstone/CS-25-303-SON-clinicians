use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use tracing::instrument;
use utoipa::ToSchema;

use crate::database::prelude::*;
#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize, Deserialize, ToSchema, Columns)]
pub struct UserLoginAttempt {
    pub id: i64,
    pub user_id: Option<i32>,
    pub ip_address: String,
    pub success: bool,
    #[schema(value_type = Option<AdditionalFootprint>)]
    pub additional_footprint: Option<Json<AdditionalFootprint>>,
    pub created_at: DateTime<FixedOffset>,
}
impl TableType for UserLoginAttempt {
    type Columns = UserLoginAttemptColumn;
    fn table_name() -> &'static str {
        "user_login_attempts"
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AdditionalFootprint {
    pub user_agent: String,
}
#[instrument]
pub async fn add_login_attempt(
    user_id: Option<i32>,
    ip_address: &str,
    success: bool,
    additional_footprint: Option<AdditionalFootprint>,
    database: &sqlx::PgPool,
) -> DBResult<()> {
    SimpleInsertQueryBuilder::new(UserLoginAttempt::table_name())
        .insert(UserLoginAttemptColumn::UserId, user_id)
        .insert(UserLoginAttemptColumn::IpAddress, ip_address)
        .insert(UserLoginAttemptColumn::Success, success)
        .insert(
            UserLoginAttemptColumn::AdditionalFootprint,
            additional_footprint.map(Json),
        )
        .query()
        .execute(database)
        .await?;
    Ok(())
}
