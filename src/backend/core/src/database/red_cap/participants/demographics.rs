use crate::database::prelude::*;
use crate::red_cap::{EducationLevel, Ethnicity, Gender, HealthInsurance, PreferredLanguage, Race};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub trait ParticipantDemograhicsType:
    for<'r> FromRow<'r, PgRow> + Unpin + Send + Sync + TableType
{
    fn get_id(&self) -> i32;
    fn columns() -> Vec<ParticipantDemograhicsColumn> {
        ParticipantDemograhicsColumn::all()
    }

    async fn find_by_participant(id: i32, database: &sqlx::PgPool) -> DBResult<Option<Self>> {
        let result = SelectQueryBuilder::with_columns(Self::table_name(), Self::columns())
            .filter(ParticipantDemograhicsColumn::ParticipantId.equals(id.value()))
            .query_as()
            .fetch_optional(database)
            .await?;
        Ok(result)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, TableType, ToSchema)]
#[table(name = "participant_demographics")]
pub struct ParticipantDemograhics {
    pub id: i32,
    /// 1:1 with [Participants]
    pub participant_id: i32,
    /// Redcap: age
    pub age: Option<i16>,
    /// Redcap Gender
    pub gender: Option<Gender>,
    /// Redcap: Race
    pub race: Option<Vec<Race>>,
    /// Not Sure???
    pub race_other: Option<String>,
    pub race_multiracial_other: Option<String>,
    /// Red Cap: ethnicity
    pub ethnicity: Option<Ethnicity>,
    pub language: Option<PreferredLanguage>,
    /// Red Cap: veteran
    /// Yes Or No
    pub is_veteran: Option<bool>,
    /// Red Cap: insurance
    pub health_insurance: Vec<HealthInsurance>,
    /// Red Cap: education
    pub highest_education_level: Option<EducationLevel>,
}

impl ParticipantDemograhicsType for ParticipantDemograhics {
    fn get_id(&self) -> i32 {
        self.id
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, FromRow)]
pub struct ParticipantDemograhicsResponse {
    pub participant_id: i32,
    /// Redcap: age
    pub age: Option<i16>,
    /// Redcap Gender
    pub gender: Option<Gender>,
    /// Redcap: Race
    pub race: Option<Vec<Race>>,
    /// Not Sure???
    pub race_other: Option<String>,
    pub race_multiracial_other: Option<String>,
    /// Red Cap: ethnicity
    pub ethnicity: Option<Ethnicity>,
    pub language: Option<PreferredLanguage>,
    /// Red Cap: veteran
    /// Yes Or No
    pub is_veteran: Option<bool>,
    /// Red Cap: insurance
    pub health_insurance: Vec<HealthInsurance>,
    /// Red Cap: education
    pub highest_education_level: Option<EducationLevel>,
}

impl TableQuery for ParticipantDemograhicsResponse {
    type Table = ParticipantDemograhics;

    fn columns() -> Vec<<Self::Table as TableType>::Columns>
    where
        Self: Sized,
    {
        vec![
            ParticipantDemograhicsColumn::ParticipantId,
            ParticipantDemograhicsColumn::Age,
            ParticipantDemograhicsColumn::Gender,
            ParticipantDemograhicsColumn::Race,
            ParticipantDemograhicsColumn::RaceOther,
            ParticipantDemograhicsColumn::RaceMultiracialOther,
            ParticipantDemograhicsColumn::Ethnicity,
            ParticipantDemograhicsColumn::Language,
            ParticipantDemograhicsColumn::IsVeteran,
            ParticipantDemograhicsColumn::HealthInsurance,
            ParticipantDemograhicsColumn::HighestEducationLevel,
        ]
    }
}

impl ParticipantDemograhicsResponse {
    pub async fn find_by_participant_id(
        participant_id: i32,
        database: &sqlx::PgPool,
    ) -> DBResult<Option<Self>> {
        SelectQueryBuilder::with_columns(ParticipantDemograhics::table_name(), Self::columns())
            .filter(ParticipantDemograhicsColumn::ParticipantId.equals(participant_id))
            .query_as()
            .fetch_optional(database)
            .await
            .map_err(Into::into)
    }
}
