use crate::database::red_cap::participants::health_overview::{
    HealthOverview, HealthOverviewColumn,
};
use crate::database::{CSPageParams, PaginatedResponse};
use chrono::NaiveDate;
use pg_extended_sqlx_queries::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use tracing::{Level, event};
use utoipa::ToSchema;

use crate::database::red_cap::case_notes::{
    CaseNote, CaseNoteColumn, CaseNoteHealthMeasures, CaseNoteHealthMeasuresColumn,
};

use crate::database::prelude::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, FromRow)]
pub struct BloodGlucoseHistory {
    /// Case Note It belongs to
    pub case_note_id: i32,
    /// Date of the visit
    pub date_of_visit: NaiveDate,
    /// Weight of the participant
    pub result: f32,
    pub fasting: Option<bool>,
}
impl BloodGlucoseHistory {
    pub async fn find_all_for_participant(
        participant_id: i32,
        page_and_size: CSPageParams,
        database: &sqlx::PgPool,
    ) -> DBResult<PaginatedResponse<BloodGlucoseHistory>> {
        let mut query = SelectQueryBuilder::new(CaseNoteHealthMeasures::table_name());
        query
            .select(CaseNoteColumn::Id.alias("case_note_id"))
            .select(CaseNoteColumn::DateOfVisit.alias("date_of_visit"))
            .select(CaseNoteHealthMeasuresColumn::GlucoseResult.alias("result"))
            .select(CaseNoteHealthMeasuresColumn::FastedAtleast2Hours.alias("fasting"))
            .select(
                SqlFunctionBuilder::count_all()
                    .then(SqlFunctionBuilder::over())
                    .alias("total_entries"),
            )
            .order_by(CaseNoteColumn::DateOfVisit, SQLOrder::Descending)
            .page_params(page_and_size)
            .join(CaseNote::table_name(), JoinType::Full, |join| {
                join.on(CaseNoteColumn::Id.equals(CaseNoteHealthMeasuresColumn::CaseNoteId))
            })
            .filter(
                CaseNoteColumn::ParticipantId
                    .equals(participant_id)
                    .and(CaseNoteHealthMeasuresColumn::GlucoseResult.is_not_null()),
            );

        let result = query.query().fetch_all(database).await?;

        let result: PaginatedResponse<BloodGlucoseHistory> =
            PaginatedResponse::from_rows(result, &page_and_size, "total_entries")?;

        Ok(result)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, FromRow)]
pub struct WeightHistory {
    /// Case Note It belongs to
    pub case_note_id: i32,
    /// Date of the visit
    pub date_of_visit: NaiveDate,
    /// Weight of the participant
    pub weight: f32,
    /// Only present if the BMI is calculated and the participant has a height
    #[sqlx(default)]
    pub bmi: Option<f64>,
}
impl WeightHistory {
    /// If page_size is 0 or less all records are returned
    pub async fn find_all_for_participant(
        participant_id: i32,
        calculate_bmi: bool,
        page_and_size: CSPageParams,
        database: &sqlx::PgPool,
    ) -> DBResult<PaginatedResponse<WeightHistory>> {
        let mut query = SelectQueryBuilder::new(CaseNoteHealthMeasures::table_name());
        query
            .select(CaseNoteColumn::Id.alias("case_note_id"))
            .select(CaseNoteColumn::DateOfVisit.alias("date_of_visit"))
            .select(CaseNoteHealthMeasuresColumn::Weight.alias("weight"))
            .select(
                SqlFunctionBuilder::count_all()
                    .then(SqlFunctionBuilder::over())
                    .alias("total_entries"),
            )
            .order_by(CaseNoteColumn::DateOfVisit, SQLOrder::Descending)
            .page_params(page_and_size)
            .join(CaseNote::table_name(), JoinType::Full, |join| {
                join.on(CaseNoteColumn::Id.equals(CaseNoteHealthMeasuresColumn::CaseNoteId))
            })
            .filter(
                CaseNoteColumn::ParticipantId
                    .equals(participant_id)
                    .and(CaseNoteHealthMeasuresColumn::Weight.is_not_null()),
            );
        if calculate_bmi {
            event!(Level::DEBUG, "Calculating BMI");
            query
                .join(HealthOverview::table_name(), JoinType::Left, |join| {
                    join.on(
                        CaseNoteColumn::ParticipantId.equals(HealthOverviewColumn::ParticipantId)
                    )
                })
                .select(
                    CaseNoteHealthMeasuresColumn::Weight
                        .multiply(703f32)
                        .divide(HealthOverviewColumn::Height.pow(2))
                        .alias("bmi"),
                );
        }
        let result: PaginatedResponse<WeightHistory> = PaginatedResponse::from_rows(
            query.query().fetch_all(database).await?,
            &page_and_size,
            "total_entries",
        )?;
        Ok(result)
    }
}
