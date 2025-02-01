use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::database::prelude::*;
#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize, Deserialize, ToSchema, Columns)]
pub struct UserLoginAttempt {
    pub id: Uuid,
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
    #[serde(default)]
    pub user_agent: String,
    #[serde(default)]
    pub request_id: String,
}
#[instrument]
pub async fn add_login_attempt(
    user_id: Option<i32>,
    ip_address: &str,
    success: bool,
    additional_footprint: Option<AdditionalFootprint>,
    database: &sqlx::PgPool,
) -> DBResult<Uuid> {
    let id = SimpleInsertQueryBuilder::new(UserLoginAttempt::table_name())
        .insert(UserLoginAttemptColumn::UserId, user_id)
        .insert(UserLoginAttemptColumn::IpAddress, ip_address)
        .insert(UserLoginAttemptColumn::Success, success)
        .insert(
            UserLoginAttemptColumn::AdditionalFootprint,
            additional_footprint.map(Json),
        )
        .return_columns(vec![UserLoginAttemptColumn::Id])
        .query_scalar()
        .fetch_one(database)
        .await?;

    Ok(id)
}
#[cfg(test)]
mod tests {

    use crate::utils::testing::config::testing::{get_testing_config, no_testing_config};

    use super::*;
    /// Tests the participant lookup query
    ///
    /// Note: This test may not find anything if the database is empty or if random data is not consistent with my setup
    #[tokio::test]
    #[ignore]
    async fn test_insert_login_attempt() -> anyhow::Result<()> {
        let Some(testing_config) = get_testing_config() else {
            no_testing_config()?;
            return Ok(());
        };
        let database = testing_config.database.connect().await?;
        let _id = add_login_attempt(
            Some(1),
            "127.0.0.1:55420",
            true,
            Some(AdditionalFootprint {
                user_agent: "test".to_string(),
                request_id: "test".to_string(),
            }),
            &database,
        )
        .await?;
        Ok(())
    }
}
