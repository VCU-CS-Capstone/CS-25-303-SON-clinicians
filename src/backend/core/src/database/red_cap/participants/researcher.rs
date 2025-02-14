use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use tabled::Tabled;
use tracing::{debug, error, event, instrument, trace, warn, Level};
use utoipa::ToSchema;

use crate::{
    database::{
        prelude::*,
        queries::{ItemOrArray, NumberQuery},
        red_cap::case_notes::{CaseNote, CaseNoteColumn},
        PaginatedResponse,
    },
    red_cap::{
        EducationLevel, Gender, HealthInsurance, PreferredLanguage, Programs, Race, SeenAtVCUHS,
        Status,
    },
};

use super::{
    ParticipantDemograhics, ParticipantDemograhicsColumn, Participants, ParticipantsColumn,
};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, FromRow, Tabled)]
pub struct ResearcherQueryResult {
    pub participant_id: i32,
    #[tabled(display = "crate::database::table_utils::display_option")]
    pub red_cap_id: Option<i32>,
    pub first_name: String,
    pub last_name: String,
    /// Phone number one
    #[tabled(display = "crate::database::table_utils::display_option")]
    pub phone_number_one: Option<String>,
    /// Second phone number
    #[tabled(display = "crate::database::table_utils::display_option")]
    pub phone_number_two: Option<String>,
    /// Other contact information
    #[tabled(display = "crate::database::table_utils::display_option")]
    pub other_contact: Option<String>,

    /// The visit history of the participant
    ///
    /// Only available if `get_visit_history` is true
    #[tabled(display = "crate::database::table_utils::count_option")]
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visit_history: Option<Vec<NaiveDate>>,
    /// The last visited date
    ///
    /// Only available if `get_last_visited` is true
    #[tabled(display = "crate::database::table_utils::display_option")]
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_visited: Option<NaiveDate>,
}

impl TableQuery for ResearcherQueryResult {
    type Table = Participants;
    fn columns() -> Vec<ParticipantsColumn> {
        vec![
            ParticipantsColumn::Id,
            ParticipantsColumn::FirstName,
            ParticipantsColumn::LastName,
        ]
    }
}
/// The researcher query
///
/// # TODO
/// - Add any of filter for Race, Gender, Education, Language, Health Insurance
/// - Mobility Devices Parameters
/// - (LOW Priority) Medication Parameters
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[schema(examples(
    ResearcherQuery::example_one,
    ResearcherQuery::example_two,
    ResearcherQuery::example_three
))]
#[serde(default)]
pub struct ResearcherQuery {
    /// Location to filter by
    pub location: Option<ItemOrArray<i32>>,
    /// Program to filter by
    pub program: Option<Programs>,
    pub vcuhs_patient_status: Option<SeenAtVCUHS>,
    /// Status to filter by
    ///
    /// By default it only returns active participants
    #[schema(default = "Active")]
    pub status: Option<Status>,

    pub gender: Option<Gender>,
    pub highest_level_of_education: Option<EducationLevel>,
    pub race: Option<Race>,
    pub language: Option<PreferredLanguage>,
    pub health_insurance: Option<HealthInsurance>,
    /// Age to filter by
    pub age: Option<NumberQuery<i16>>,
    /// Get the participants visit history
    ///
    /// Capped at 10
    pub get_visit_history: bool,
    /// Get the last visited date
    pub get_last_visited: bool,
}
impl ResearcherQuery {
    fn example_one() -> Self {
        Self {
            location: Some(ItemOrArray::Item(1)),
            program: Some(Programs::MHWP),
            get_visit_history: true,
            ..Default::default()
        }
    }
    fn example_two() -> Self {
        Self {
            location: Some(ItemOrArray::Array(vec![1, 2])),
            get_last_visited: true,
            ..Default::default()
        }
    }
    fn example_three() -> Self {
        Self {
            age: Some(NumberQuery::GreaterThan(18)),
            ..Default::default()
        }
    }
}
impl Default for ResearcherQuery {
    fn default() -> Self {
        Self {
            location: None,
            program: None,
            vcuhs_patient_status: None,
            status: Some(Status::Active),
            gender: None,
            highest_level_of_education: None,
            //race: None,
            language: None,
            health_insurance: None,
            age: None,
            get_visit_history: false,
            get_last_visited: false,
            race: None,
        }
    }
}
impl ResearcherQuery {
    #[instrument(skip(database))]
    pub async fn query(
        self,
        page_and_size: PageParams,
        database: &PgPool,
    ) -> Result<PaginatedResponse<ResearcherQueryResult>, DBError> {
        let Self {
            location,
            program,
            vcuhs_patient_status,
            status,
            gender,
            highest_level_of_education,
            language,
            health_insurance,
            age,
            get_visit_history,
            get_last_visited,
            race,
        } = self;
        if get_last_visited && get_visit_history {
            warn!(
                "get_last_visited and get_visit_history are both true. This is really unnecessary"
            );
        }
        let mut query = SelectQueryBuilder::with_columns(
            Participants::table_name(),
            vec![
                ParticipantsColumn::RedCapId.dyn_column(),
                ParticipantsColumn::FirstName.dyn_column(),
                ParticipantsColumn::LastName.dyn_column(),
                ParticipantsColumn::PhoneNumberOne.dyn_column(),
                ParticipantsColumn::PhoneNumberTwo.dyn_column(),
                ParticipantsColumn::OtherContact.dyn_column(),
            ],
        );
        query.select(ParticipantsColumn::Id.alias("participant_id"));

        query
            .join(
                ParticipantDemograhics::table_name(),
                JoinType::Inner,
                |join| {
                    join.on(
                        ParticipantsColumn::Id.equals(ParticipantDemograhicsColumn::ParticipantId)
                    )
                },
            )
            .total_count("total")
            .page_params(page_and_size);

        if let Some(location) = location {
            match location {
                ItemOrArray::Item(item) => {
                    query.filter(ParticipantsColumn::Location.equals(item.value()))
                }
                ItemOrArray::Array(items) => {
                    query.filter(ParticipantsColumn::Location.equals(items.value().any()))
                }
            };
        }
        if let Some(program) = program {
            query.filter(ParticipantsColumn::Program.equals(program.value()));
        }
        if let Some(vcuhs_patient_status) = vcuhs_patient_status {
            query.filter(
                ParticipantsColumn::VcuhsPatientStatus.equals(vcuhs_patient_status.value()),
            );
        }
        if let Some(status) = status {
            query.filter(ParticipantsColumn::Status.equals(status.value()));
        }

        if let Some(age) = age {
            query.filter(age.filter(ParticipantDemograhicsColumn::Age));
        };
        if let Some(gender) = gender {
            query.filter(ParticipantDemograhicsColumn::Gender.equals(gender.value()));
        }
        if let Some(highest_level_of_education) = highest_level_of_education {
            query.filter(
                ParticipantDemograhicsColumn::HighestEducationLevel
                    .equals(highest_level_of_education.value()),
            );
        }
        if let Some(race) = race {
            query.filter(ParticipantDemograhicsColumn::Race.equals(race.value()));
        }
        if let Some(language) = language {
            query.filter(ParticipantDemograhicsColumn::Language.equals(language.value()));
        }
        if let Some(health_insurance) = health_insurance {
            query.filter(
                ParticipantDemograhicsColumn::HealthInsurance.equals(health_insurance.value()),
            );
        }
        if get_visit_history {
            trace!("Getting Visit History");
            query.select(
                SelectExprBuilder::new(CaseNote::table_name())
                    .column(CaseNoteColumn::DateOfVisit)
                    .limit(10)
                    .filter(
                        CaseNoteColumn::ParticipantId.equals(ParticipantsColumn::Id.dyn_column()),
                    )
                    .order_by(CaseNoteColumn::DateOfVisit, SQLOrder::Descending)
                    .array()
                    .alias("visit_history"),
            );
        }
        if get_last_visited {
            trace!("Getting Last Visited Date");
            query.select(
                SelectExprBuilder::new(CaseNote::table_name())
                    .column(CaseNoteColumn::DateOfVisit)
                    .limit(1)
                    .filter(
                        CaseNoteColumn::ParticipantId.equals(ParticipantsColumn::Id.dyn_column()),
                    )
                    .order_by(CaseNoteColumn::DateOfVisit, SQLOrder::Descending)
                    .alias("last_visited"),
            );
        }
        let mut total_count: Option<i64> = None;
        let result = query.query().fetch_all(database).await?;
        let mut resulting_items = Vec::with_capacity(result.len());
        for item in result {
            if total_count.is_none() {
                let total_count_value = item.try_get("total");
                match total_count_value {
                    Err(err) => {
                        error!(?err, "Failed to get total count");
                    }
                    Ok(ok) => {
                        debug!(?ok, "Got total count");
                        total_count = Some(ok);
                    }
                }
            }
            let item = ResearcherQueryResult::from_row(&item)?;
            resulting_items.push(item);
        }
        event!(
            Level::TRACE,
            ?resulting_items,
            ?total_count,
            "Returning result"
        );
        let result = PaginatedResponse::create_response(
            resulting_items,
            &page_and_size,
            total_count.unwrap_or(0),
        );
        Ok(result)
    }
}
#[cfg(test)]
mod tests {

    use tabled::Table;

    use crate::utils::testing::config::testing::{get_testing_config, no_testing_config};

    use super::*;
    /// Test the examples of the researcher query
    #[tokio::test]
    #[ignore]
    async fn test_examples() -> anyhow::Result<()> {
        let Some(config) = get_testing_config() else {
            no_testing_config()?;
            return Ok(());
        };
        config.init_logger();
        let database = config.connect_to_db().await?;
        let query: Vec<ResearcherQuery> = vec![
            ResearcherQuery::example_one(),
            ResearcherQuery::example_two(),
            ResearcherQuery::example_three(),
        ];

        for query in query {
            let result = query
                .clone()
                .query(
                    PageParams {
                        page_number: 1,
                        page_size: 10,
                    },
                    &database,
                )
                .await
                .expect("Failed to Execute Researcher Query");

            if result.is_empty() {
                eprintln!("No participant found. But it might be expected");
                continue;
            }
            println!("Found {} participants from {:?}", result.len(), query);
            println!("{}", Table::new(result.iter()));
        }

        Ok(())
    }
    #[ignore]
    #[tokio::test]
    async fn more_tests() -> anyhow::Result<()> {
        let Some(config) = get_testing_config() else {
            no_testing_config()?;
            return Ok(());
        };
        config.init_logger();
        let database = config.connect_to_db().await?;
        let query: Vec<ResearcherQuery> = vec![ResearcherQuery {
            age: Some(">25".parse().unwrap()),
            gender: Some(Gender::Male),
            ..Default::default()
        }];

        for query in query {
            let result = query
                .clone()
                .query(
                    PageParams {
                        page_number: 1,
                        page_size: 10,
                    },
                    &database,
                )
                .await
                .expect("Failed to Execute Researcher Query");

            if result.is_empty() {
                eprintln!("No participant found. But it might be expected");
                continue;
            }
            println!("Found {} participants from {:?}", result.len(), query);
            println!("{}", Table::new(result.iter()));
        }

        Ok(())
    }
}
