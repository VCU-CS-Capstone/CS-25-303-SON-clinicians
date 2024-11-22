use crate::{
    database::prelude::*,
    red_cap::{
        converter::participants::{
            RedCapHealthOverview, RedCapParticipant, RedCapParticipantDemographics,
        },
        DegreeLevel, Ethnicity, Gender, HealthInsurance, PreferredLanguage, Programs, Race, Status,
    },
};
use chrono::{DateTime, FixedOffset};
use cs25_303_macros::Columns;
use serde::{Deserialize, Serialize};
pub mod goals;
pub mod health_overview;
mod lookup;
mod medications;
mod new;
pub use lookup::*;
pub use medications::*;
pub use new::*;
use sqlx::{postgres::PgRow, prelude::FromRow};
use tracing::error;
pub trait ParticipantType: for<'r> FromRow<'r, PgRow> + Unpin + Send + Sync {
    fn get_id(&self) -> i32;

    /// Leaving this to the default implementation is not recommended. As it will return all columns
    fn columns() -> Vec<ParticipantsColumn> {
        ParticipantsColumn::all()
    }

    async fn find_by_id(id: i32, database: &sqlx::PgPool) -> DBResult<Option<Self>>
    where
        Self: TableType,
    {
        let result = SimpleSelectQueryBuilder::new(Self::table_name(), &Self::columns())
            .where_equals(ParticipantsColumn::Id, id)
            .query_as()
            .fetch_optional(database)
            .await?;
        Ok(result)
    }
    async fn find_by_red_cap_id(red_cap_id: i32, database: &sqlx::PgPool) -> DBResult<Option<Self>>
    where
        Self: TableType,
    {
        let result = SimpleSelectQueryBuilder::new(Self::table_name(), &Self::columns())
            .where_equals(ParticipantsColumn::RedCapId, red_cap_id)
            .query_as()
            .fetch_optional(database)
            .await?;
        Ok(result)
    }
}
/// Database Table: `participants`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, Columns)]
pub struct Participants {
    pub id: i32,
    /// The ID within Red Cap. This is separate so if we added creating a new participant
    /// We know what users have been added to redcap or not
    pub red_cap_id: Option<i32>,
    /// Redcap: first_name
    pub first_name: String,
    /// Red Cap last_name
    pub last_name: String,
    /// RedCap: phone1
    pub phone_number_one: Option<String>,
    /// RedCap: phone2
    pub phone_number_two: Option<String>,
    /// RedCap: other_info
    pub other_contact: Option<String>,
    pub program: Programs,
    /// Redcap: rhwp_location
    /// Relates to [super::Locations]
    pub location: Option<i32>,
    /// Red Cap: pt_status
    pub status: Option<Status>,
    /// Red Cap: behav_health_risk
    pub behavioral_risks_identified: Option<String>,
    /// Red Cap: consent_cc
    pub date_care_coordination_consent_signed: Option<chrono::NaiveDate>,
    /// Red Cap: consent_home
    pub date_home_visit_consent_signed: Option<chrono::NaiveDate>,
    /// Red CAp: date_intake
    pub signed_up_on: chrono::NaiveDate,
    /// For Database Only
    pub added_to_db_at: DateTime<FixedOffset>,
    /// For Database Only
    pub last_synced_with_redcap: Option<DateTime<FixedOffset>>,
}
impl Participants {
    pub async fn set_red_cap_id(
        &mut self,
        red_cap_id: Option<i32>,
        db: &sqlx::PgPool,
    ) -> DBResult<()> {
        self.red_cap_id = red_cap_id;
        sqlx::query("UPDATE participants SET red_cap_id = $1 WHERE id = $2")
            .bind(red_cap_id)
            .bind(self.id)
            .execute(db)
            .await?;
        Ok(())
    }
    #[tracing::instrument]
    pub async fn update_from_red_cap(
        &mut self,
        red_cap_participant: RedCapParticipant,
        red_cap_demographics: RedCapParticipantDemographics,
        red_cap_health_overview: RedCapHealthOverview,
        _db: &sqlx::PgPool,
    ) -> DBResult<()> {
        //TODO: Implement
        error!("Not Implemented");
        Ok(())
    }
}
impl TableType for Participants {
    type Columns = ParticipantsColumn;
    fn table_name() -> &'static str {
        "participants"
    }
}
impl ParticipantType for Participants {
    fn get_id(&self) -> i32 {
        self.id
    }
}

pub trait ParticipantDemograhicsType:
    for<'r> FromRow<'r, PgRow> + Unpin + Send + Sync + TableType
{
    fn get_id(&self) -> i32;
    fn columns() -> Vec<ParticipantDemograhicsColumn> {
        ParticipantDemograhicsColumn::all()
    }

    async fn find_by_participant(id: i32, database: &sqlx::PgPool) -> DBResult<Option<Self>> {
        let result = SimpleSelectQueryBuilder::new(Self::table_name(), &Self::columns())
            .where_equals(ParticipantDemograhicsColumn::ParticipantId, id)
            .query_as()
            .fetch_optional(database)
            .await?;
        Ok(result)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, Columns)]
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
    pub race_multiple: Option<String>,
    /// Red Cap: ethnicity
    pub ethnicity: Option<Ethnicity>,
    pub language: Option<PreferredLanguage>,
    /// Red Cap: veteran
    /// Yes Or No
    pub is_veteran: Option<bool>,
    /// Red Cap: insurance
    pub health_insurance: Vec<HealthInsurance>,
    /// Red Cap: education
    pub highest_education_level: Option<DegreeLevel>,
}
impl TableType for ParticipantDemograhics {
    type Columns = ParticipantDemograhicsColumn;
    fn table_name() -> &'static str {
        "participant_demographics"
    }
}
impl ParticipantDemograhicsType for ParticipantDemograhics {
    fn get_id(&self) -> i32 {
        self.id
    }
}
