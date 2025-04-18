use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use utoipa::ToSchema;

use crate::database::{
    CSPageParams, PaginatedResponse,
    prelude::*,
    red_cap::{
        Locations, LocationsColumn,
        participants::{
            Participants, ParticipantsColumn,
            goals::{ParticipantGoals, ParticipantGoalsColumn},
        },
    },
};

use super::DebugParticipantSummary;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ParticipantsWithNoGoals {
    #[serde(default)]
    pub get_location_name: bool,
}

impl ParticipantsWithNoGoals {
    pub async fn execute(
        &self,
        page_and_size: CSPageParams,
        executor: impl Executor<'_, Database = Postgres>,
    ) -> Result<PaginatedResponse<DebugParticipantSummary>, DBError> {
        let mut query = SelectQueryBuilder::new(Participants::table_name());
        query
            .select(ParticipantsColumn::Id)
            .select(ParticipantsColumn::FirstName)
            .select(ParticipantsColumn::LastName)
            .select(ParticipantsColumn::Program)
            .select(ParticipantsColumn::Location)
            .select(
                SqlFunctionBuilder::count_all()
                    .then(SqlFunctionBuilder::over())
                    .alias("total_entries"),
            )
            .page_params(page_and_size)
            .filter(
                SelectExprBuilder::new(ParticipantGoals::table_name())
                    .select_expr(SqlFunctionBuilder::count_all())
                    .filter(ParticipantGoalsColumn::ParticipantId.equals(ParticipantsColumn::Id))
                    .equals(0),
            );
        if self.get_location_name {
            query
                .join(Locations::table_name(), JoinType::Left, |join| {
                    join.on(ParticipantsColumn::Location.equals(LocationsColumn::Id))
                })
                .select(LocationsColumn::Name.alias("location_name"));
        }

        let result: PaginatedResponse<DebugParticipantSummary> = PaginatedResponse::from_rows(
            query.query().fetch_all(executor).await?,
            &page_and_size,
            "total_entries",
        )?;
        Ok(result)
    }
}
#[cfg(test)]
mod tests {

    use crate::{
        database::{CSPageParams, red_cap::debug_reports::goals::ParticipantsWithNoGoals},
        utils::testing::config::testing::{get_testing_config, no_testing_config},
    };

    #[ignore]
    #[tokio::test]
    async fn test_query() -> anyhow::Result<()> {
        let Some(config) = get_testing_config() else {
            no_testing_config()?;
            return Ok(());
        };
        config.init_logger();
        let database = config.connect_to_db().await?;

        let query = ParticipantsWithNoGoals {
            get_location_name: true,
        };

        let result = query.execute(CSPageParams::default(), &database).await?;
        for participant in result.data {
            println!("{:?}", participant);
        }
        Ok(())
    }
}
