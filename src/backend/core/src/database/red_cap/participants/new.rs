use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::Executor;

use crate::{
    database::DBResult,
    red_cap::{
        DegreeLevel, Ethnicity, Gender, HealthInsurance, MobilityDevice, PreferredLanguage,
        Programs, Race, Status,
    },
};

use super::{
    health_overview::{HealthOverview, HealthOverviewColumn},
    DBError, ParticipantDemograhics, ParticipantDemograhicsColumn, Participants,
    ParticipantsColumn, SimpleInsertQueryBuilder, TableType,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewParticipant {
    pub red_cap_id: Option<i32>,
    pub first_name: String,
    pub last_name: String,
    pub phone_number_one: Option<String>,
    pub phone_number_two: Option<String>,
    pub other_contact: Option<String>,
    pub program: Programs,
    pub location: Option<i32>,
    pub status: Option<Status>,
    pub behavioral_risks_identified: Option<String>,
    pub date_care_coordination_consent_signed: Option<chrono::NaiveDate>,
    pub date_home_visit_consent_signed: Option<chrono::NaiveDate>,
    pub signed_up_on: chrono::NaiveDate,
    pub last_synced_with_redcap: Option<DateTime<FixedOffset>>,
}

impl NewParticipant {
    pub async fn insert_return_participant(
        self,
        database: impl Executor<'_, Database = sqlx::Postgres>,
    ) -> DBResult<Participants> {
        let Self {
            red_cap_id,
            first_name,
            last_name,
            phone_number_one,
            phone_number_two,
            other_contact,
            program,
            location,
            status,
            behavioral_risks_identified,
            date_care_coordination_consent_signed,
            date_home_visit_consent_signed,
            signed_up_on,
            last_synced_with_redcap,
        } = self;

        let new_participant = SimpleInsertQueryBuilder::new(Participants::table_name())
            .insert(ParticipantsColumn::RedCapId, red_cap_id)
            .insert(ParticipantsColumn::FirstName, first_name)
            .insert(ParticipantsColumn::LastName, last_name)
            .insert(ParticipantsColumn::PhoneNumberOne, phone_number_one)
            .insert(ParticipantsColumn::PhoneNumberTwo, phone_number_two)
            .insert(ParticipantsColumn::OtherContact, other_contact)
            .insert(ParticipantsColumn::Program, program)
            .insert(ParticipantsColumn::Location, location)
            .insert(ParticipantsColumn::Status, status)
            .insert(
                ParticipantsColumn::BehavioralRisksIdentified,
                behavioral_risks_identified,
            )
            .insert(
                ParticipantsColumn::DateCareCoordinationConsentSigned,
                date_care_coordination_consent_signed,
            )
            .insert(
                ParticipantsColumn::DateHomeVisitConsentSigned,
                date_home_visit_consent_signed,
            )
            .insert(ParticipantsColumn::SignedUpOn, signed_up_on)
            .insert(
                ParticipantsColumn::LastSyncedWithRedcap,
                last_synced_with_redcap,
            )
            .return_all()
            .query_as::<Participants>()
            .fetch_one(database)
            .await?;
        Ok(new_participant)
    }
}
#[derive(Debug, Clone, Deserialize, Default)]
pub struct NewDemographics {
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
impl NewDemographics {
    pub async fn insert_none(
        self,
        participant_id: i32,
        database: impl Executor<'_, Database = sqlx::Postgres>,
    ) -> DBResult<()> {
        let Self {
            age,
            gender,
            race,
            race_other,
            race_multiple,
            ethnicity,
            language,
            is_veteran,
            health_insurance,
            highest_education_level,
        } = self;

        SimpleInsertQueryBuilder::new(ParticipantDemograhics::table_name())
            .insert(ParticipantDemograhicsColumn::ParticipantId, participant_id)
            .insert(ParticipantDemograhicsColumn::Age, age)
            .insert(ParticipantDemograhicsColumn::Gender, gender)
            .insert(ParticipantDemograhicsColumn::Race, race)
            .insert(ParticipantDemograhicsColumn::RaceOther, race_other)
            .insert(ParticipantDemograhicsColumn::RaceMultiple, race_multiple)
            .insert(ParticipantDemograhicsColumn::Ethnicity, ethnicity)
            .insert(ParticipantDemograhicsColumn::Language, language)
            .insert(ParticipantDemograhicsColumn::IsVeteran, is_veteran)
            .insert(
                ParticipantDemograhicsColumn::HealthInsurance,
                health_insurance,
            )
            .insert(
                ParticipantDemograhicsColumn::HighestEducationLevel,
                highest_education_level,
            )
            .query()
            .execute(database)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct NewHealthOverview {
    pub height: Option<i32>,
    /// Red Cap: health_conditions
    pub reported_health_conditions: Option<String>,
    /// Red Cap: allergies
    pub allergies: Option<String>,
    /// Red Cap: personal_cuff
    pub has_blood_pressure_cuff: Option<bool>,
    /// Red Cap: num_meds
    pub takes_more_than_5_medications: Option<bool>,

    pub mobility_devices: Option<Vec<MobilityDevice>>,
}
impl NewHealthOverview {
    pub async fn insert_return_health_overview(
        self,
        participant_id: i32,
        database: impl Executor<'_, Database = sqlx::Postgres>,
    ) -> DBResult<HealthOverview> {
        let Self {
            height,
            reported_health_conditions,
            allergies,
            has_blood_pressure_cuff,
            takes_more_than_5_medications,
            mobility_devices,
        } = self;

        SimpleInsertQueryBuilder::new(HealthOverview::table_name())
            .insert(HealthOverviewColumn::ParticipantId, participant_id)
            .insert(HealthOverviewColumn::Height, height)
            .insert(
                HealthOverviewColumn::ReportedHealthConditions,
                reported_health_conditions,
            )
            .insert(HealthOverviewColumn::Allergies, allergies)
            .insert(
                HealthOverviewColumn::HasBloodPressureCuff,
                has_blood_pressure_cuff,
            )
            .insert(
                HealthOverviewColumn::TakesMoreThan5Medications,
                takes_more_than_5_medications,
            )
            .insert(HealthOverviewColumn::MobilityDevices, mobility_devices)
            .return_all()
            .query_as::<HealthOverview>()
            .fetch_one(database)
            .await
            .map_err(DBError::from)
    }
    pub async fn insert_return_none(
        self,
        participant_id: i32,
        database: impl Executor<'_, Database = sqlx::Postgres>,
    ) -> DBResult<()> {
        let Self {
            height,
            reported_health_conditions,
            allergies,
            has_blood_pressure_cuff,
            takes_more_than_5_medications,
            ..
        } = self;

        SimpleInsertQueryBuilder::new(HealthOverview::table_name())
            .insert(HealthOverviewColumn::ParticipantId, participant_id)
            .insert(HealthOverviewColumn::Height, height)
            .insert(
                HealthOverviewColumn::ReportedHealthConditions,
                reported_health_conditions,
            )
            .insert(HealthOverviewColumn::Allergies, allergies)
            .insert(
                HealthOverviewColumn::HasBloodPressureCuff,
                has_blood_pressure_cuff,
            )
            .insert(
                HealthOverviewColumn::TakesMoreThan5Medications,
                takes_more_than_5_medications,
            )
            .query()
            .execute(database)
            .await?;
        // TODO: Insert mobility devices - We can't do this due to the executor moving issue
        Ok(())
    }
}
