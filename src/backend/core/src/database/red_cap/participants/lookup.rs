use std::fmt::Debug;

use crate::database::{
    prelude::*,
    red_cap::case_notes::{CaseNote, CaseNoteColumn},
};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use tabled::Tabled;
use tracing::instrument;
use utoipa::ToSchema;

use crate::red_cap::Programs;

use super::{ParticipantType, Participants, ParticipantsColumn};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, Tabled, ToSchema)]
pub struct ParticipantLookup {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    #[tabled(display_with = "crate::database::table_utils::display_option")]
    pub phone_number_one: Option<String>,
    #[tabled(display_with = "crate::database::table_utils::display_option")]
    pub phone_number_two: Option<String>,
    pub program: Programs,
    #[tabled(display_with = "crate::database::table_utils::display_option")]
    pub location: Option<i32>,
    #[tabled(display_with = "crate::database::table_utils::display_option")]
    #[sqlx(default)]
    pub last_visited: Option<NaiveDate>,
}

impl ParticipantType for ParticipantLookup {
    fn get_id(&self) -> i32 {
        self.id
    }

    fn columns() -> Vec<ParticipantsColumn> {
        vec![
            ParticipantsColumn::Id,
            ParticipantsColumn::FirstName,
            ParticipantsColumn::LastName,
            ParticipantsColumn::PhoneNumberOne,
            ParticipantsColumn::PhoneNumberTwo,
            ParticipantsColumn::Program,
            ParticipantsColumn::Location,
        ]
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, ToSchema)]
#[serde(default)]
pub struct ParticipantLookupQuery {
    /// First name to filter by
    pub first_name: String,
    /// Last name to filter by
    pub last_name: String,
    /// Location to filter by
    pub location: Option<i32>,
    /// Program to filter by
    pub program: Option<Programs>,
    /// Rather or not to pull the participants last visited date
    #[serde(default)]
    pub get_last_visited: bool,
}
impl ParticipantLookupQuery {
    pub fn apply_arguments<'args, Q>(&self, query: &mut Q)
    where
        Q: WhereableTool<'args>,
    {
        let Self {
            first_name,
            last_name,
            location,
            program,
            ..
        } = self;
        query.where_column(ParticipantsColumn::FirstName.lower(), |builder| {
            builder
                .like(format!("{}%", first_name.to_lowercase()))
                .build()
        });
        if !last_name.is_empty() {
            query.where_column(ParticipantsColumn::LastName.lower(), |builder| {
                builder
                    .like(format!("{}%", last_name.to_lowercase()))
                    .build()
            });
        }
        if let Some(location) = location {
            query.where_column(ParticipantsColumn::Location, |builder| {
                builder.equals(*location).build()
            });
        }
        if let Some(program) = program {
            query.where_column(ParticipantsColumn::Program, |builder| {
                builder.equals(*program).build()
            });
        }
    }
    #[instrument(name = "ParticipantLookupQuery::find", skip(database))]
    pub async fn find(
        self,
        page_and_size: impl Into<PageParams> + Debug,
        database: &PgPool,
    ) -> DBResult<PaginatedResponse<ParticipantLookup>> {
        let page_params: PageParams = page_and_size.into();
        let mut query = SimpleSelectQueryBuilderV2::new(
            Participants::table_name(),
            ParticipantLookup::columns(),
        );

        self.apply_arguments(&mut query);
        if self.get_last_visited {
            query.select_also(CaseNote::table_name(), |mut builder| {
                builder
                    .column(CaseNoteColumn::DateOfVisit)
                    .where_column(CaseNoteColumn::ParticipantId, |builder| {
                        builder.equals_column(ParticipantsColumn::Id).build()
                    })
                    .order_by(CaseNoteColumn::DateOfVisit, SQLOrder::Descending)
                    .build_as("last_visited")
            });
        }
        query.page_params(page_params);
        #[cfg(test)]
        {
            let sql = query.sql();
            tracing::debug!("SQL: {}", sql);
        }
        let total: i64 = {
            let mut count = SelectCount::new(Participants::table_name());
            self.apply_arguments(&mut count);
            count.query_scalar().fetch_one(database).await?
        };
        let result: Vec<ParticipantLookup> = query.query_as().fetch_all(database).await?;
        let result = page_params.create_result(total, result);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {

    use tabled::Table;

    use crate::database::red_cap::participants::health_overview::{
        HealthOverview, HealthOverviewType,
    };

    use super::*;
    /// Tests the participant lookup query
    ///
    /// Note: This test may not find anything if the database is empty or if random data is not consistent with my setup
    #[tokio::test]
    #[ignore]
    async fn test_participant_lookup_query() -> anyhow::Result<()> {
        crate::test_utils::init_logger();
        let database = crate::database::tests::connect_to_db().await?;
        let query: Vec<ParticipantLookupQuery> = vec![
            ParticipantLookupQuery {
                first_name: "John".to_string(),
                last_name: String::new(),
                get_last_visited: true,
                ..Default::default()
            },
            ParticipantLookupQuery {
                first_name: "Hannah".to_string(),
                last_name: "H".to_string(),
                program: Some(Programs::RHWP),
                get_last_visited: true,

                ..Default::default()
            },
            ParticipantLookupQuery {
                first_name: "Hannah".to_string(),
                last_name: "H".to_string(),
                program: Some(Programs::MHWP),
                location: Some(9),
                get_last_visited: true,

                ..Default::default()
            },
        ];

        for query in query {
            let result = query.clone().find((10, 100), &database).await.unwrap();
            if result.is_empty() {
                eprintln!("No participant found. But it might be expected");
                continue;
            }
            println!("Found {} participants from {:?}", result.len(), query);
            let table = Table::new(result.iter()).to_string();
            println!("{}", table);
            let participant = result.first().unwrap();
            let health_overiew =
                HealthOverview::find_by_participant_id(participant.id, &database).await?;
            println!("Health Overview: {:?}", health_overiew);
        }

        Ok(())
    }
}
