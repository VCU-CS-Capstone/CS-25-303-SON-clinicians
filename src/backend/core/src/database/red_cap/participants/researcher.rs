use std::fmt::Debug;

use crate::{
    database::{
        PaginatedResponse,
        prelude::*,
        queries::{ItemOrArray, NumberQuery, array::ArrayQuery},
        red_cap::{
            case_notes::{
                BloodPressureType, CaseNote, CaseNoteColumn, CaseNoteHealthMeasures,
                CaseNoteHealthMeasuresColumn, HealthMeasureBloodPressure,
                HealthMeasureBloodPressureColumn,
            },
            participants::health_overview::{HealthOverview, HealthOverviewColumn},
        },
    },
    red_cap::{
        EducationLevel, Gender, HealthInsurance, PreferredLanguage, Programs, Race, SeenAtVCUHS,
        Status,
    },
};
use pg_extended_sqlx_queries::pagination::{
    PageParams, PaginationOwnedSupportingTool, PaginationSupportingTool,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, prelude::FromRow};
use tabled::Tabled;
use tracing::{Level, Span, error, event, instrument, trace, warn};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use utoipa::ToSchema;

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, Default)]
pub struct ResearcherQueryBloodPressure {
    #[serde(alias = "type", default)]
    pub reading_type: BloodPressureType,
    pub systolic: Option<NumberQuery<i16>>,
    pub diastolic: Option<NumberQuery<i16>>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, Default)]
pub struct ResearcherQueryGlucose {
    pub glucose: NumberQuery<f32>,
    /// Undefined will tell the query you do not want to filter by this
    pub fasted_atleast_2_hours: Option<bool>,
}
/// The researcher query
///
/// # TODO
/// - Add any of filter for Race, Gender, Education, Language, Health Insurance
/// - Mobility Devices Parameters
/// - (LOW Priority) Medication Parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
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
    pub race: Option<ArrayQuery<Race>>,
    pub language: Option<PreferredLanguage>,
    pub health_insurance: Option<ArrayQuery<HealthInsurance>>,
    /// Age to filter by
    pub age: Option<NumberQuery<i16>>,
    /// Get the participants visit history
    ///
    /// Capped at 10
    pub get_visit_history: bool,
    /// Get the last visited date
    pub get_last_visited: bool,

    /// BMI Query
    pub bmi: Option<NumberQuery<f32>>,
    /// Blood Pressure Query
    pub blood_pressure: Option<ResearcherQueryBloodPressure>,
    /// Glucose Query
    pub glucose: Option<ResearcherQueryGlucose>,
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
            bmi: None,
            blood_pressure: None,
            glucose: None,
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
        let span = Span::current();
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
            bmi,
            blood_pressure,
            glucose,
        } = self;
        // TODO: Improve this. This is very messy
        let mut query = if let Some(ResearcherQueryBloodPressure {
            reading_type,
            systolic,
            diastolic,
        }) =
            blood_pressure.filter(|bp| bp.systolic.is_some() || bp.diastolic.is_some())
        {
            let mut query = SelectQueryBuilder::new(HealthMeasureBloodPressure::table_name());

            query
                .distinct()
                .join(
                    CaseNoteHealthMeasures::table_name(),
                    JoinType::Full,
                    |join| {
                        join.on(CaseNoteHealthMeasuresColumn::Id
                            .equals(HealthMeasureBloodPressureColumn::HealthMeasureId))
                    },
                )
                .join(CaseNote::table_name(), JoinType::Full, |join| {
                    join.on(CaseNoteColumn::Id.equals(CaseNoteHealthMeasuresColumn::CaseNoteId))
                })
                .join(Participants::table_name(), JoinType::Full, |join| {
                    join.on(ParticipantsColumn::Id.equals(CaseNoteColumn::ParticipantId))
                })
                .filter(HealthMeasureBloodPressureColumn::BloodPressureType.equals(reading_type));
            if let Some(systolic) = systolic {
                query.filter(systolic.filter(HealthMeasureBloodPressureColumn::Systolic));
            }
            if let Some(diastolic) = diastolic {
                query.filter(diastolic.filter(HealthMeasureBloodPressureColumn::Diastolic));
            }
            query
        } else if bmi.is_some() || glucose.is_some() {
            let mut query = SelectQueryBuilder::new(CaseNoteHealthMeasures::table_name());
            query
                .distinct()
                .join(CaseNote::table_name(), JoinType::Full, |join| {
                    join.on(CaseNoteColumn::Id.equals(CaseNoteHealthMeasuresColumn::CaseNoteId))
                })
                .join(Participants::table_name(), JoinType::Full, |join| {
                    join.on(ParticipantsColumn::Id.equals(CaseNoteColumn::ParticipantId))
                })
                .filter(
                    CaseNoteHealthMeasuresColumn::Weight
                        .is_not_null()
                        .and(HealthOverviewColumn::Height.is_not_null()),
                );
            query
        } else {
            SelectQueryBuilder::new(Participants::table_name())
        };
        if get_last_visited && get_visit_history {
            warn!(
                "get_last_visited and get_visit_history are both true. This is really unnecessary"
            );
        }
        query
            .select(ParticipantsColumn::Id.alias("participant_id"))
            .select_many(vec![
                ParticipantsColumn::RedCapId.dyn_column(),
                ParticipantsColumn::FirstName.dyn_column(),
                ParticipantsColumn::LastName.dyn_column(),
                ParticipantsColumn::PhoneNumberOne.dyn_column(),
                ParticipantsColumn::PhoneNumberTwo.dyn_column(),
                ParticipantsColumn::OtherContact.dyn_column(),
            ])
            .join(
                ParticipantDemograhics::table_name(),
                JoinType::Inner,
                |join| {
                    join.on(
                        ParticipantsColumn::Id.equals(ParticipantDemograhicsColumn::ParticipantId)
                    )
                },
            )
            .join(HealthOverview::table_name(), JoinType::Inner, |join| {
                join.on(
                    HealthOverviewColumn::ParticipantId.equals(ParticipantsColumn::Id.dyn_column())
                )
            })
            .select(
                SqlFunctionBuilder::count_all()
                    .then(SqlFunctionBuilder::over())
                    .alias("total"),
            )
            .page_params(page_and_size);

        if let Some(bmi) = bmi {
            query.filter(
                bmi.complex_value_filter(
                    CaseNoteHealthMeasuresColumn::Weight
                        .multiply(703f32)
                        .divide(HealthOverviewColumn::Height.pow(2)),
                ),
            );
        }
        if let Some(glucose) = glucose {
            query.filter(
                glucose
                    .glucose
                    .filter(CaseNoteHealthMeasuresColumn::GlucoseResult),
            );
            if let Some(fasted) = glucose.fasted_atleast_2_hours {
                query.filter(CaseNoteHealthMeasuresColumn::FastedAtleast2Hours.equals(fasted));
            }
        }

        if let Some(location) = location {
            match location {
                ItemOrArray::Item(item) => query.filter(ParticipantsColumn::Location.equals(item)),
                ItemOrArray::Array(items) => {
                    query.filter(ParticipantsColumn::Location.equals(items.any()))
                }
            };
        }
        if let Some(program) = program {
            query.filter(ParticipantsColumn::Program.equals(program));
        }
        if let Some(vcuhs_patient_status) = vcuhs_patient_status {
            query.filter(ParticipantsColumn::VcuhsPatientStatus.equals(vcuhs_patient_status));
        }
        if let Some(status) = status {
            query.filter(ParticipantsColumn::Status.equals(status));
        }

        if let Some(age) = age {
            query.filter(age.filter(ParticipantDemograhicsColumn::Age));
        };
        if let Some(gender) = gender {
            query.filter(ParticipantDemograhicsColumn::Gender.equals(gender));
        }
        if let Some(highest_level_of_education) = highest_level_of_education {
            query.filter(
                ParticipantDemograhicsColumn::HighestEducationLevel
                    .equals(highest_level_of_education),
            );
        }
        if let Some(race) = race.filter(|race| race.len() > 0) {
            query.filter(race.filter(ParticipantDemograhicsColumn::Race));
        }
        if let Some(language) = language {
            query.filter(ParticipantDemograhicsColumn::Language.equals(language));
        }
        if let Some(health_insurance) = health_insurance.filter(|race| race.len() > 0) {
            query.filter(health_insurance.filter(ParticipantDemograhicsColumn::HealthInsurance));
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
        let query = match query.query().fetch_all(database).await {
            Ok(ok) => ok,
            Err(err) => {
                event!(Level::ERROR, ?err, "Failed to execute query");
                span.set_status(opentelemetry::trace::Status::error(err.to_string()));
                return Err(err.into());
            }
        };
        let result = PaginatedResponse::from_rows(query, &page_and_size, "total")?;
        Ok(result)
    }
}
#[cfg(test)]
mod tests {

    use pg_extended_sqlx_queries::pagination::PageParams;
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
            execute_query(query, &database).await?;
        }

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_blood_pressure() -> anyhow::Result<()> {
        let Some(config) = get_testing_config() else {
            no_testing_config()?;
            return Ok(());
        };
        config.init_logger();
        let database = config.connect_to_db().await?;
        let query: Vec<ResearcherQuery> = vec![ResearcherQuery {
            blood_pressure: Some(ResearcherQueryBloodPressure {
                reading_type: BloodPressureType::Sit,
                systolic: Some(">=120".parse().unwrap()),
                diastolic: Some(">=80".parse().unwrap()),
            }),
            ..Default::default()
        }];

        for query in query {
            execute_query(query, &database).await?;
        }

        Ok(())
    }
    #[ignore]
    #[tokio::test]
    async fn test_bmi() -> anyhow::Result<()> {
        let Some(config) = get_testing_config() else {
            no_testing_config()?;
            return Ok(());
        };
        config.init_logger();
        let database = config.connect_to_db().await?;
        let query: Vec<ResearcherQuery> = vec![ResearcherQuery {
            bmi: Some(">=25".parse().unwrap()),
            ..Default::default()
        }];

        for query in query {
            execute_query(query, &database).await?;
        }

        Ok(())
    }
    #[ignore]
    #[tokio::test]
    async fn test_glucose() -> anyhow::Result<()> {
        let Some(config) = get_testing_config() else {
            no_testing_config()?;
            return Ok(());
        };
        config.init_logger();
        let database = config.connect_to_db().await?;
        let query: Vec<ResearcherQuery> = vec![
            ResearcherQuery {
                glucose: Some(ResearcherQueryGlucose {
                    glucose: ">=100".parse().unwrap(),
                    fasted_atleast_2_hours: Some(true),
                }),
                ..Default::default()
            },
            ResearcherQuery {
                glucose: Some(ResearcherQueryGlucose {
                    glucose: ">=100".parse().unwrap(),
                    fasted_atleast_2_hours: None,
                }),
                ..Default::default()
            },
        ];

        for query in query {
            execute_query(query, &database).await?;
        }

        Ok(())
    }
    #[ignore]
    #[tokio::test]
    async fn bmi_bp_and_glucose() -> anyhow::Result<()> {
        let Some(config) = get_testing_config() else {
            no_testing_config()?;
            return Ok(());
        };
        config.init_logger();
        let database = config.connect_to_db().await?;
        let query: Vec<ResearcherQuery> = vec![ResearcherQuery {
            bmi: Some(">=25".parse().unwrap()),
            blood_pressure: Some(ResearcherQueryBloodPressure {
                reading_type: BloodPressureType::Sit,
                systolic: Some(">=120".parse().unwrap()),
                diastolic: Some(">=80".parse().unwrap()),
            }),
            glucose: Some(ResearcherQueryGlucose {
                glucose: ">=100".parse().unwrap(),
                fasted_atleast_2_hours: Some(true),
            }),
            get_visit_history: true,
            get_last_visited: true,
            ..Default::default()
        }];

        for query in query {
            execute_query(query, &database).await?;
        }

        Ok(())
    }
    async fn execute_query(query: ResearcherQuery, database: &PgPool) -> anyhow::Result<()> {
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
            return Ok(());
        }
        println!("Found {} participants from {:?}", result.len(), query);
        println!("{}", Table::new(result.iter()));
        Ok(())
    }
}
