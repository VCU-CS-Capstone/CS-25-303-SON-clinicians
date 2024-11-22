pub mod new;
use std::fmt::Debug;

use crate::database::prelude::*;
use crate::red_cap::converter::case_notes::{
    OtherCaseNoteData, RedCapCaseNoteBase, RedCapHealthMeasures,
};
use crate::{
    database::tools::{SimpleSelectQueryBuilder, TableType},
    red_cap::VisitType,
};
use chrono::{DateTime, FixedOffset, NaiveDate};
use new::NewBloodPressure;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::EnumIter;
use tracing::error;
use utoipa::ToSchema;
pub mod questions;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow, Columns)]
pub struct CaseNote {
    pub id: i32,
    /// Relates to #[crate::database::red_cap::participants::Participants]
    pub participant_id: i32,
    /// Relates to #[crate::database::red_cap::Locations]
    ///
    /// RWHP Red Cap ID: `rhwp_location_visit`
    /// MHWP Red Cap ID: `mhwp_location_visit`
    ///
    /// Petersburg Sub Red Cap ID: `mhwp_location_visit_petersburg`
    pub location: Option<i32>,
    /// Red Cap ID: `visit_type`
    pub visit_type: Option<VisitType>,
    /// Redcap ID: exit_age
    pub age: Option<i16>,
    /// Red Cap ID: `reason`
    pub reason_for_visit: Option<String>,
    /// Red Cap ID: subjective_info
    pub info_provided_by_caregiver: Option<String>,
    /// Red Cap ID: visit_date
    pub date_of_visit: NaiveDate,
    /// Whether the case note is completed
    pub completed: bool,
    /// DATABASE ONLY
    pub pushed_to_red_cap: bool,
    /// Instance Number of the case note
    pub red_cap_instance: Option<i32>,
    /// DATABASE ONLY
    pub last_synced_with_redcap: Option<DateTime<FixedOffset>>,
    /// DATABASE ONLY
    pub created_at: DateTime<FixedOffset>,
}
impl TableType for CaseNote {
    type Columns = CaseNoteColumn;
    fn table_name() -> &'static str {
        "case_notes"
    }
}
impl CaseNote {
    pub async fn find_by_participant_id_and_redcap_instance(
        participant_id: i32,
        redcap_instance: i32,
        database: &sqlx::PgPool,
    ) -> DBResult<Option<Self>> {
        let result = sqlx::query_as(
            "
            SELECT * FROM case_notes
            WHERE participant_id = $1 AND red_cap_instance = $2
            ",
        )
        .bind(participant_id)
        .bind(redcap_instance)
        .fetch_optional(database)
        .await?;
        Ok(result)
    }
    pub async fn find_by_id(id: i32, database: &sqlx::PgPool) -> DBResult<Option<Self>> {
        let result = sqlx::query_as(
            "
            SELECT * FROM case_notes
            WHERE id = $1
            ",
        )
        .bind(id)
        .fetch_optional(database)
        .await?;
        Ok(result)
    }
    pub async fn find_by_participant_id(
        participant_id: i32,
        database: &sqlx::PgPool,
    ) -> DBResult<Vec<Self>> {
        let result = sqlx::query_as(
            "
            SELECT * FROM case_notes
            WHERE participant_id = $1
            ",
        )
        .bind(participant_id)
        .fetch_all(database)
        .await?;
        Ok(result)
    }
    pub async fn update_instance_id(
        &self,
        instance_id: i32,
        database: &sqlx::PgPool,
    ) -> DBResult<()> {
        sqlx::query(
            "
            UPDATE case_notes
            SET redcap_instance = $1
            WHERE id = $2
            ",
        )
        .bind(instance_id)
        .bind(self.id)
        .execute(database)
        .await?;
        Ok(())
    }
    #[tracing::instrument()]
    pub async fn update_from_red_cap(
        &self,
        case_note: RedCapCaseNoteBase,
        health_measures: RedCapHealthMeasures,
        other: OtherCaseNoteData,
        database: &sqlx::PgPool,
    ) -> DBResult<()> {
        error!("Not Implemented");
        //TODO: Implement
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow, Columns)]
pub struct CaseNoteHealthMeasures {
    pub id: i32,
    /// 1:1 with [CaseNote]
    pub case_note_id: i32,
    /// Weight Taken RED Cap ID: weight_yn
    /// Weight Red Cap: weight
    pub weight: Option<f32>,
    /// Redcap ID: glucose_yn
    pub glucose_tested: bool,
    /// Redcap ID: glucose
    pub glucose_result: Option<f32>,
    /// Redcap ID: glucose_fasting
    ///
    /// ## RedCap Values
    /// - 2: Yes
    /// - 1: No
    pub fasted_atleast_2_hours: Option<bool>,
    ///Function, Assistive Devices, and/or Limitations to ADLs/IADLs
    /// Redcap ID: visit_function
    pub other: Option<String>,
}
impl CaseNoteHealthMeasures {
    pub async fn add_bp(&self, bp: NewBloodPressure, db: &PgPool) -> DBResult<()> {
        SimpleInsertQueryBuilder::new(HealthMeasureBloodPressure::table_name())
            .insert(HealthMeasureBloodPressureColumn::HealthMeasureId, self.id)
            .insert(
                HealthMeasureBloodPressureColumn::BloodPressureType,
                bp.blood_pressure_type,
            )
            .insert(HealthMeasureBloodPressureColumn::Systolic, bp.systolic)
            .insert(HealthMeasureBloodPressureColumn::Diastolic, bp.diastolic)
            .query()
            .execute(db)
            .await?;
        Ok(())
    }
}
impl TableType for CaseNoteHealthMeasures {
    type Columns = CaseNoteHealthMeasuresColumn;
    fn table_name() -> &'static str {
        "case_note_health_measures"
    }
}
impl CaseNoteHealthMeasures {
    pub async fn find_by_id(id: i32, database: &sqlx::PgPool) -> DBResult<Option<Self>> {
        let result = sqlx::query_as(
            "
            SELECT * FROM case_note_health_measures
            WHERE id = $1
            ",
        )
        .bind(id)
        .fetch_optional(database)
        .await?;
        Ok(result)
    }
    pub async fn find_by_case_note_id(
        case_note_id: i32,
        database: &sqlx::PgPool,
    ) -> DBResult<Option<Self>> {
        SimpleSelectQueryBuilder::new(Self::table_name(), &CaseNoteHealthMeasuresColumn::all())
            .where_equals(CaseNoteHealthMeasuresColumn::CaseNoteId, case_note_id)
            .query_as()
            .fetch_optional(database)
            .await
            .map_err(DBError::from)
    }
}
#[derive(Clone, PartialEq, Serialize, Deserialize, Type, ToSchema, EnumIter)]
#[sqlx(type_name = "VARCHAR")]
pub enum BloodPressureType {
    Sit,
    /// Orthostatic Blood Pressure
    Stand,
    /// Only Used if HealthOverview is marked as person having a blood pressure cuff
    Personal,
}
impl Debug for BloodPressureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            BloodPressureType::Sit => "Sit",
            BloodPressureType::Stand => "Stand",
            BloodPressureType::Personal => "Personal",
        };
        write!(
            f,
            "{}{{systolic = {}, diastolic ={}, yn = {}}}",
            value,
            self.systolic(),
            self.diastolic(),
            self.yes_or_no_question()
        )
    }
}
impl BloodPressureType {
    /// The systolic value for the blood pressure
    pub fn systolic(&self) -> &'static str {
        match self {
            BloodPressureType::Sit => "bp_sit_syst",
            BloodPressureType::Stand => "bp_stand_syst",
            BloodPressureType::Personal => "cuff_systolic",
        }
    }
    /// The diasolic value for the blood pressure
    pub fn diastolic(&self) -> &'static str {
        match self {
            BloodPressureType::Sit => "bp_sit_dia",
            BloodPressureType::Stand => "bp_stand_dia",
            BloodPressureType::Personal => "cuff_diastolic",
        }
    }
    /// The was read question id in red_cap
    ///
    /// We ignore this when retrieving the data
    /// because it is not needed
    ///
    /// However, it is needed when pushing data to red_cap
    pub fn yes_or_no_question(&self) -> &'static str {
        match self {
            BloodPressureType::Sit => "bp_sit",
            BloodPressureType::Stand => "bp_stand",
            BloodPressureType::Personal => "cuff_systolic",
        }
    }
}
/// Blood Pressure gets its own table because it happens between 0-3 different ways
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow, Columns)]
pub struct HealthMeasureBloodPressure {
    pub id: i32,
    /// Each [CaseNote] can have at most 3 blood pressures
    pub health_measure_id: i32,
    /// The Type of Blood Pressure
    pub blood_pressure_type: BloodPressureType,
    /// Possible Red CAP IDs: bp_sit_syst, bp_stand_syst
    pub systolic: i16,
    /// Possible Red CAP IDs: bp_sit_dia, bp_stand_dia
    pub diastolic: i16,
}

impl TableType for HealthMeasureBloodPressure {
    type Columns = HealthMeasureBloodPressureColumn;
    fn table_name() -> &'static str {
        "health_measure_blood_pressure"
    }
}
