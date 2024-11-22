use crate::database::prelude::*;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use tabled::Tabled;
use tracing::{debug, instrument};

use crate::red_cap::Programs;

use super::{ParticipantType, ParticipantsColumn};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, Tabled)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Builder)]
pub struct ParticipantLookupQuery {
    pub first_name: String,
    pub last_name: String,
    #[builder(setter(into, strip_option), default)]
    pub location: Option<i32>,
    #[builder(setter(into, strip_option), default)]
    pub program: Option<Programs>,
    #[builder(setter(into, strip_option), default)]
    pub limit: Option<i64>,
}
impl ParticipantLookupQuery {
    #[instrument(name = "ParticipantLookupQuery::find", skip(database))]
    pub async fn find(self, database: &PgPool) -> DBResult<Vec<ParticipantLookup>> {
        let Self {
            first_name,
            last_name,
            location,
            program,
            ..
        } = self;
        let mut query =
            SimpleSelectQueryBuilder::new("participants", &ParticipantLookup::columns());
        query.where_like_then(
            ParticipantsColumn::FirstName.lower(),
            format!("%{}%", first_name.to_lowercase()),
            |query_where| {
                query_where.and_like(
                    ParticipantsColumn::LastName.lower(),
                    format!("%{}%", last_name.to_lowercase()),
                );
                if let Some(location) = location {
                    query_where.and_equals(ParticipantsColumn::Location, location);
                }
                if let Some(program) = program {
                    query_where.and_equals(ParticipantsColumn::Program, program);
                }
            },
        );

        if let Some(limit) = self.limit {
            query.limit(limit);
        }
        if tracing::enabled!(tracing::Level::DEBUG) {
            let query = query.sql();
            debug!(?query, "Executing Query");
        }
        let result = query.query_as().fetch_all(database).await?;

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
        let database = crate::database::tests::setup_query_test().await?;
        let query: Vec<ParticipantLookupQuery> = vec![
            ParticipantLookupQuery {
                first_name: "Wyatt".to_string(),
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
                limit: Some(5),
            },
        ];

        for query in query {
            let result = query.clone().find(&database).await.unwrap();
            if result.is_empty() {
                eprintln!("No participant found. But it might be expected");
                return Ok(());
            }
            println!("Found {} participants from {:?}", result.len(), query);
            let table = Table::new(&result).to_string();
            println!("{}", table);
            let participant = result.first().unwrap();
            let health_overiew =
                HealthOverview::find_by_participant_id(participant.id, &database).await?;
            println!("Health Overview: {:?}", health_overiew);
        }

        Ok(())
    }
}
