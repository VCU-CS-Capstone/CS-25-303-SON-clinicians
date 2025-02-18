pub mod new;
pub mod queries;
use std::fmt::Debug;

use crate::database::{prelude::*, PaginatedResponse};
use crate::red_cap::converter::case_notes::{
    OtherCaseNoteData, RedCapCaseNoteBase, RedCapHealthMeasures,
};
use crate::red_cap::VisitType;
use chrono::{DateTime, FixedOffset, NaiveDate};
use new::NewBloodPressure;
use pg_extended_sqlx_queries::pagination::{PageParams, PaginationSupportingTool};
use pg_extended_sqlx_queries::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::EnumIter;
use tracing::{debug, error, instrument};
use utoipa::ToSchema;

pub mod questions;

pub trait CaseNoteType:
    for<'r> FromRow<'r, PgRow> + Unpin + Send + Sync + TableQuery<Table = CaseNote>
{
    fn get_id(&self) -> i32;

    /// Find a case note by its ID
    async fn find_by_id(id: i32, database: &sqlx::PgPool) -> DBResult<Option<Self>> {
        let result = SelectQueryBuilder::with_columns(CaseNote::table_name(), Self::columns())
            .filter(CaseNoteColumn::Id.equals(id.value()))
            .query_as()
            .fetch_optional(database)
            .await?;
        Ok(result)
    }
    /// Get all case notes for a participant
    async fn get_all_by_participant_id(
        participant_id: i32,
        database: &sqlx::PgPool,
    ) -> DBResult<Vec<Self>> {
        let result = SelectQueryBuilder::with_columns(CaseNote::table_name(), Self::columns())
            .filter(CaseNoteColumn::ParticipantId.equals(participant_id.value()))
            .order_by(CaseNoteColumn::DateOfVisit, SQLOrder::Descending)
            .query_as()
            .fetch_all(database)
            .await?;
        Ok(result)
    }
    #[instrument]
    async fn fetch_paginated_by_participant_id(
        participant_id: i32,
        page_params: PageParams,
        database: &sqlx::PgPool,
    ) -> DBResult<PaginatedResponse<Self>> {
        let count = {
            SelectCount::new(CaseNote::table_name())
                .filter(CaseNoteColumn::ParticipantId.equals(participant_id.value()))
                .execute(database)
                .await?
        };
        if count == 0 {
            debug!(?page_params, "No case notes found found");
            return Ok(PaginatedResponse::default());
        }

        if count < page_params.offset() as i64 {
            debug!(?page_params, ?count, "The offset os greater than the count");
            return Ok(PaginatedResponse::default());
        }
        let query_result =
            SelectQueryBuilder::with_columns(CaseNote::table_name(), Self::columns())
                .filter(CaseNoteColumn::ParticipantId.equals(participant_id.value()))
                .page_params(page_params)
                .order_by(CaseNoteColumn::DateOfVisit, SQLOrder::Descending)
                .query_as()
                .fetch_all(database)
                .await?;
        Ok(PaginatedResponse::create_response(
            query_result,
            &page_params,
            count,
        ))
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow, TableType, ToSchema)]
#[table(name = "case_notes")]
pub struct CaseNote {
    pub id: i32,
    /// Relates to #[crate::database::red_cap::participants::Participants]
    pub participant_id: i32,
    /// Relates to #[crate::database::red_cap::Locations]
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
    pub last_synced_with_red_cap: Option<DateTime<FixedOffset>>,
    /// DATABASE ONLY
    pub created_at: DateTime<FixedOffset>,
}

impl CaseNote {
    pub async fn find_by_participant_id_and_redcap_instance(
        participant_id: i32,
        redcap_instance: i32,
        database: &sqlx::PgPool,
    ) -> DBResult<Option<Self>> {
        let result = SelectQueryBuilder::with_columns(Self::table_name(), Self::columns())
            .filter(
                CaseNoteColumn::ParticipantId
                    .equals(participant_id.value())
                    .and(CaseNoteColumn::RedCapInstance.equals(redcap_instance.value())),
            )
            .query_as()
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
        let result = SelectQueryBuilder::with_columns(Self::table_name(), Self::columns())
            .filter(CaseNoteColumn::ParticipantId.equals(participant_id.value()))
            .order_by(CaseNoteColumn::DateOfVisit, SQLOrder::Descending)
            .query_as()
            .fetch_all(database)
            .await?;
        Ok(result)
    }
    pub async fn update_instance_id(
        &self,
        instance_id: i32,
        database: &sqlx::PgPool,
    ) -> DBResult<()> {
        let result = UpdateQueryBuilder::new(Self::table_name())
            .set(CaseNoteColumn::RedCapInstance, instance_id.value())
            .set(
                CaseNoteColumn::LastSyncedWithRedCap,
                SqlFunctionBuilder::now(),
            )
            .filter(CaseNoteColumn::Id.equals(self.id.value()))
            .query()
            .execute(database)
            .await?;
        if result.rows_affected() != 1 {
            error!(?result, "Failed to update case note instance id");
        }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow, TableType)]
#[table(name = "case_note_health_measures")]
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
        InsertQueryBuilder::new(HealthMeasureBloodPressure::table_name())
            .insert(HealthMeasureBloodPressureColumn::HealthMeasureId, self.id)
            .insert(
                HealthMeasureBloodPressureColumn::BloodPressureType,
                bp.blood_pressure_type.value(),
            )
            .insert(HealthMeasureBloodPressureColumn::Systolic, bp.systolic)
            .insert(HealthMeasureBloodPressureColumn::Diastolic, bp.diastolic)
            .query()
            .execute(db)
            .await?;
        Ok(())
    }
    pub async fn add_many_bp(&self, bp: Vec<NewBloodPressure>, db: &PgPool) -> DBResult<()> {
        let mut query = InsertManyBuilder::new(
            HealthMeasureBloodPressure::table_name(),
            vec![
                HealthMeasureBloodPressureColumn::HealthMeasureId,
                HealthMeasureBloodPressureColumn::BloodPressureType,
                HealthMeasureBloodPressureColumn::Systolic,
                HealthMeasureBloodPressureColumn::Diastolic,
            ],
        );
        // On Conflict we will update the values
        query.set_on_conflict(OnConflict {
            conflict_target: ConflictTarget::columns(vec![
                HealthMeasureBloodPressureColumn::HealthMeasureId.dyn_column(),
                HealthMeasureBloodPressureColumn::BloodPressureType.dyn_column(),
            ]),
            action: ConflictAction::DoUpdate(
                vec![
                    HealthMeasureBloodPressureColumn::Systolic,
                    HealthMeasureBloodPressureColumn::Diastolic,
                ]
                .into_iter()
                .map(|column| SetColumm::from(column))
                .collect(),
            ),
        });

        for bp in bp {
            query.insert_row_ordered(|row| {
                row.insert(self.id)
                    .insert(bp.blood_pressure_type.value())
                    .insert(bp.systolic)
                    .insert(bp.diastolic);
            });
        }

        query.query().execute(db).await?;
        Ok(())
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
        SelectQueryBuilder::with_columns(Self::table_name(), CaseNoteHealthMeasuresColumn::all())
            .filter(CaseNoteHealthMeasuresColumn::CaseNoteId.equals(case_note_id.value()))
            .query_as()
            .fetch_optional(database)
            .await
            .map_err(DBError::from)
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type, ToSchema, EnumIter)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow, TableType)]
#[table(name = "health_measure_blood_pressure")]
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
