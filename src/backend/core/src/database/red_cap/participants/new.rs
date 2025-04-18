use crate::{database::prelude::*, red_cap::SeenAtVCUHS};
use serde::{Deserialize, Serialize};

use crate::{
    database::DBResult,
    red_cap::{
        EducationLevel, Ethnicity, Gender, HealthInsurance, MobilityDevice, PreferredLanguage,
        Programs, Race, Status,
    },
};

use super::{
    DBError, ParticipantDemograhics, ParticipantDemograhicsColumn, Participants,
    ParticipantsColumn, TableType,
    health_overview::{HealthOverview, HealthOverviewColumn},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NewParticipant {
    pub red_cap_id: Option<i32>,
    pub first_name: String,
    pub last_name: String,
    pub phone_number_one: Option<String>,
    pub phone_number_two: Option<String>,
    pub other_contact: Option<String>,
    pub program: Programs,
    pub vcuhs_patient_status: Option<SeenAtVCUHS>,
    pub location: Option<i32>,
    pub status: Option<Status>,
    pub behavioral_risks_identified: Option<String>,
    pub date_care_coordination_consent_signed: Option<NaiveDate>,
    pub date_home_visit_consent_signed: Option<NaiveDate>,
    pub signed_up_on: NaiveDate,
    pub last_synced_with_redcap: Option<DateTime<FixedOffset>>,
}

impl NewParticipant {
    pub async fn insert_returning(
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
            vcuhs_patient_status,
            status,
            behavioral_risks_identified,
            date_care_coordination_consent_signed,
            date_home_visit_consent_signed,
            signed_up_on,
            last_synced_with_redcap,
        } = self;

        let new_participant = InsertQueryBuilder::new(Participants::table_name())
            .insert(ParticipantsColumn::RedCapId, red_cap_id.value())
            .insert(ParticipantsColumn::FirstName, first_name.value())
            .insert(ParticipantsColumn::LastName, last_name.value())
            .insert(ParticipantsColumn::PhoneNumberOne, phone_number_one.value())
            .insert(ParticipantsColumn::PhoneNumberTwo, phone_number_two.value())
            .insert(ParticipantsColumn::OtherContact, other_contact.value())
            .insert(ParticipantsColumn::Program, program.value())
            .insert(
                ParticipantsColumn::VcuhsPatientStatus,
                vcuhs_patient_status.value(),
            )
            .insert(ParticipantsColumn::Location, location.value())
            .insert(ParticipantsColumn::Status, status.value())
            .insert(
                ParticipantsColumn::BehavioralRisksIdentified,
                behavioral_risks_identified.value(),
            )
            .insert(
                ParticipantsColumn::DateCareCoordinationConsentSigned,
                date_care_coordination_consent_signed.value(),
            )
            .insert(
                ParticipantsColumn::DateHomeVisitConsentSigned,
                date_home_visit_consent_signed.value(),
            )
            .insert(ParticipantsColumn::SignedUpOn, signed_up_on.value())
            .insert(
                ParticipantsColumn::LastSyncedWithRedCap,
                last_synced_with_redcap.value(),
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
impl NewDemographics {
    pub async fn insert(
        self,
        participant_id: i32,
        database: impl Executor<'_, Database = sqlx::Postgres>,
    ) -> DBResult<()> {
        let Self {
            age,
            gender,
            race,
            race_other,
            race_multiracial_other,
            ethnicity,
            language,
            is_veteran,
            health_insurance,
            highest_education_level,
        } = self;

        InsertQueryBuilder::new(ParticipantDemograhics::table_name())
            .insert(
                ParticipantDemograhicsColumn::ParticipantId,
                participant_id.value(),
            )
            .insert(ParticipantDemograhicsColumn::Age, age.value())
            .insert(ParticipantDemograhicsColumn::Gender, gender.value())
            .insert(ParticipantDemograhicsColumn::Race, race.value())
            .insert(ParticipantDemograhicsColumn::RaceOther, race_other.value())
            .insert(
                ParticipantDemograhicsColumn::RaceMultiracialOther,
                race_multiracial_other.value(),
            )
            .insert(ParticipantDemograhicsColumn::Ethnicity, ethnicity.value())
            .insert(ParticipantDemograhicsColumn::Language, language.value())
            .insert(ParticipantDemograhicsColumn::IsVeteran, is_veteran.value())
            .insert(
                ParticipantDemograhicsColumn::HealthInsurance,
                health_insurance.value(),
            )
            .insert(
                ParticipantDemograhicsColumn::HighestEducationLevel,
                highest_education_level.value(),
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
    /// Red Cap: mobility_devices
    pub mobility_devices: Option<Vec<MobilityDevice>>,
}
impl NewHealthOverview {
    fn insert_base(self, participant_id: i32) -> InsertQueryBuilder<'static> {
        let Self {
            height,
            reported_health_conditions,
            allergies,
            has_blood_pressure_cuff,
            takes_more_than_5_medications,
            mobility_devices,
        } = self;

        let mut builder = InsertQueryBuilder::new(HealthOverview::table_name());

        builder
            .insert(HealthOverviewColumn::ParticipantId, participant_id.value())
            .insert(HealthOverviewColumn::Height, height.value())
            .insert(
                HealthOverviewColumn::ReportedHealthConditions,
                reported_health_conditions.value(),
            )
            .insert(HealthOverviewColumn::Allergies, allergies.value())
            .insert(
                HealthOverviewColumn::HasBloodPressureCuff,
                has_blood_pressure_cuff.value(),
            )
            .insert(
                HealthOverviewColumn::TakesMoreThan5Medications,
                takes_more_than_5_medications.value(),
            )
            .insert(
                HealthOverviewColumn::MobilityDevices,
                mobility_devices.value(),
            );
        builder
    }
    pub async fn insert_returning(
        self,
        participant_id: i32,
        database: impl Executor<'_, Database = sqlx::Postgres>,
    ) -> DBResult<HealthOverview> {
        self.insert_base(participant_id)
            .return_all()
            .query_as::<HealthOverview>()
            .fetch_one(database)
            .await
            .map_err(DBError::from)
    }
    pub async fn insert(
        self,
        participant_id: i32,
        database: impl Executor<'_, Database = sqlx::Postgres>,
    ) -> DBResult<()> {
        self.insert_base(participant_id)
            .query()
            .execute(database)
            .await?;
        Ok(())
    }
}
