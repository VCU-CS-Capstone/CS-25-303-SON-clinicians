use crate::database::prelude::*;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use tabled::Tabled;
use tracing::{debug, instrument};
use utoipa::ToSchema;

use crate::red_cap::Programs;

use super::{ParticipantType, ParticipantsColumn};

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Builder, ToSchema)]
pub struct ParticipantLookupQuery {
    pub first_name: String,
    pub last_name: String,
    #[builder(setter(into, strip_option), default)]
    pub location: Option<i32>,
    #[builder(setter(into, strip_option), default)]
    pub program: Option<Programs>,
}
impl ParticipantLookupQuery {
    #[instrument(name = "ParticipantLookupQuery::find", skip(database))]
    pub async fn find(
        self,
        database: &PgPool,
        page: i32,
        page_size: i32,
    ) -> DBResult<PaginatedResponse<ParticipantLookup>> {
        let Self {
            first_name,
            last_name,
            location,
            program,
            ..
        } = self;
        let mut query =
            SimpleSelectQueryBuilderV2::new("participants", ParticipantLookup::columns());
        query.where_column(ParticipantsColumn::FirstName.lower(), |builder| {
            builder
                .like(format!("%{}%", first_name.to_lowercase()))
                .build()
        });
        if !last_name.is_empty() {
            query.where_column(ParticipantsColumn::LastName.lower(), |builder| {
                builder
                    .like(format!("%{}%", last_name.to_lowercase()))
                    .build()
            });
        }
        if let Some(location) = location {
            query.where_column(ParticipantsColumn::Location, |builder| {
                builder.equals(location).build()
            });
        }
        if let Some(program) = program {
            query.where_column(ParticipantsColumn::Program, |builder| {
                builder.equals(program).build()
            });
        }
        #[cfg(test)]
        {
            let sql = query.sql();
            debug!("SQL: {}", sql);
        }
        if page >0{
            query.offset(page * page_size);
        }
        query.limit(page_size);

        let result: Vec<ParticipantLookup> = query.query_as().fetch_all(database).await?;
        let result = PaginatedResponse {
            page: 0,
            page_size: 0,
            total: 0,
            data: result,
        };
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
                ..Default::default()
            },
            ParticipantLookupQuery {
                first_name: "Hannah".to_string(),
                last_name: "H".to_string(),
                program: Some(Programs::RHWP),
                ..Default::default()
            },
            ParticipantLookupQuery {
                first_name: "Hannah".to_string(),
                last_name: "H".to_string(),
                program: Some(Programs::MHWP),
                location: Some(9),
            },
        ];

        for query in query {
            let result = query.clone().find(&database, 0, 100).await.unwrap();
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
