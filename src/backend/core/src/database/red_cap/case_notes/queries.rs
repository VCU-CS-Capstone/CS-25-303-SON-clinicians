use ahash::{HashMap, HashMapExt};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::{FromRow, Type},
    Decode,
};
use utoipa::ToSchema;

use crate::red_cap::VisitType;

use super::{BloodPressureType, CaseNoteType, DBResult};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, FromRow)]
pub struct CaseNoteIDAndDate {
    /// Case Note ID
    pub id: i32,
    /// Date of the visit
    pub date_of_visit: NaiveDate,
}

impl CaseNoteType for CaseNoteIDAndDate {
    fn get_id(&self) -> i32 {
        self.id
    }
    fn columns() -> Vec<super::CaseNoteColumn> {
        vec![
            super::CaseNoteColumn::Id,
            super::CaseNoteColumn::DateOfVisit,
        ]
    }
}
/// A small struct to represent a case note for listing visits
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, FromRow)]
pub struct CaseNoteListItem {
    /// Case Note ID
    pub id: i32,
    /// Participant ID
    pub participant_id: i32,
    /// Location of the visit
    pub location: Option<i32>,
    /// Visit Type
    pub visit_type: Option<VisitType>,
    /// Date of the visit
    pub date_of_visit: NaiveDate,
}
impl CaseNoteType for CaseNoteListItem {
    fn get_id(&self) -> i32 {
        self.id
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
}
impl WeightHistory {
    pub async fn find_all_for_participant(
        participant_id: i32,
        database: &sqlx::PgPool,
    ) -> DBResult<Vec<Self>> {
        let result = sqlx::query_as(
            "
                SELECT case_notes.id as case_note_id, case_notes.date_of_visit, cnhm.weight FROM case_notes
                    FULL JOIN case_note_health_measures cnhm on case_notes.id = cnhm.case_note_id
                    WHERE case_notes.participant_id = $1 AND cnhm.weight IS NOT NULL
                    ORDER BY case_notes.date_of_visit DESC",
        )
        .bind(participant_id)
        .fetch_all(database)
        .await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct BloodPressureHistory {
    /// Case Note It belongs to
    pub case_note_id: i32,
    pub date_of_visit: NaiveDate,
    pub blood_pressure: HashMap<BloodPressureType, BloodPressureReading>,
}

impl<'a, R: sqlx::Row> sqlx::FromRow<'a, R> for BloodPressureHistory
where
    &'a str: sqlx::ColumnIndex<R>,
    i32: Decode<'a, R::Database> + Type<R::Database>,
    NaiveDate: Decode<'a, R::Database> + Type<R::Database>,
    Vec<BloodPressureHistoryItem>: Decode<'a, R::Database> + Type<R::Database>,
{
    fn from_row(row: &'a R) -> ::sqlx::Result<Self> {
        let case_note_id: i32 = row.try_get("case_note_id")?;
        let date_of_visit: NaiveDate = row.try_get("date_of_visit")?;
        let blood_pressure: Vec<BloodPressureHistoryItem> = row.try_get("blood_pressure")?;
        let mut readings = HashMap::with_capacity(blood_pressure.len());

        for bp in blood_pressure {
            readings.insert(
                bp.blood_pressure_type,
                BloodPressureReading {
                    systolic: bp.systolic,
                    diastolic: bp.diastolic,
                },
            );
        }

        Ok(BloodPressureHistory {
            case_note_id,
            date_of_visit,
            blood_pressure: readings,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, Type)]
#[sqlx(type_name = "RECORD")]
pub struct BloodPressureHistoryItem {
    pub blood_pressure_type: BloodPressureType,
    pub systolic: i16,
    pub diastolic: i16,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, Type)]
#[sqlx(type_name = "RECORD")]
pub struct BloodPressureReading {
    pub systolic: i16,
    pub diastolic: i16,
}
impl BloodPressureHistory {
    pub async fn find_all_for_participant(
        participant_id: i32,
        database: &sqlx::PgPool,
    ) -> DBResult<Vec<Self>> {
        let query = sqlx::query_as("
        SELECT case_notes.id as case_note_id, case_notes.date_of_visit as date_of_visit,
            ARRAY(
                SELECT (BP.blood_pressure_type,BP.systolic, BP.diastolic) FROM health_measure_blood_pressure AS BP
                    WHERE BP.health_measure_id = HM.id
            ) as blood_pressure
            from case_note_health_measures as HM
            FULL JOIN case_notes ON case_notes.id = HM.case_note_id
            WHERE case_notes.participant_id = $1;")
            .bind(participant_id).fetch_all(database).await?;
        Ok(query)
    }
}

#[cfg(test)]

mod tests {
    #[tokio::test]
    pub async fn bp_test() -> anyhow::Result<()> {
        let pool = crate::database::tests::connect_to_db().await?;
        let bps = super::BloodPressureHistory::find_all_for_participant(1, &pool).await?;
        println!("{:?}", bps);
        Ok(())
    }
}
