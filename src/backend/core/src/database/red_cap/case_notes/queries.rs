use ahash::HashMap;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::{FromRow, Type},
    Decode,
};
use utoipa::ToSchema;

use crate::{database::tools::TableQuery, red_cap::VisitType};

use super::{BloodPressureType, CaseNoteType, DBResult, PageParams, PaginatedResponse};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, FromRow)]
pub struct CaseNoteIDAndDate {
    /// Case Note ID
    pub id: i32,
    /// Date of the visit
    pub date_of_visit: NaiveDate,
}
impl TableQuery for CaseNoteIDAndDate {
    type Table = super::CaseNote;

    fn columns() -> Vec<super::CaseNoteColumn> {
        vec![
            super::CaseNoteColumn::Id,
            super::CaseNoteColumn::DateOfVisit,
        ]
    }
}

impl CaseNoteType for CaseNoteIDAndDate {
    fn get_id(&self) -> i32 {
        self.id
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
impl TableQuery for CaseNoteListItem {
    type Table = super::CaseNote;
    fn columns() -> Vec<super::CaseNoteColumn> {
        vec![
            super::CaseNoteColumn::Id,
            super::CaseNoteColumn::ParticipantId,
            super::CaseNoteColumn::Location,
            super::CaseNoteColumn::VisitType,
            super::CaseNoteColumn::DateOfVisit,
        ]
    }
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
    /// If page_size is 0 or less all records are returned
    pub async fn find_all_for_participant(
        participant_id: i32,
        page_and_size: impl Into<PageParams>,
        database: &sqlx::PgPool,
    ) -> DBResult<PaginatedResponse<WeightHistory>> {
        let page_and_size: PageParams = page_and_size.into();
        let offset_and_limit = if page_and_size.page_size > 0 {
            format!(
                "LIMIT {} OFFSET {}",
                page_and_size.page_size,
                page_and_size.offset()
            )
        } else {
            "".to_string()
        };

        let data = sqlx::query_as(
         &format!(
            "
            SELECT case_notes.id as case_note_id, case_notes.date_of_visit, cnhm.weight FROM case_notes
                FULL JOIN case_note_health_measures cnhm on case_notes.id = cnhm.case_note_id
                WHERE case_notes.participant_id = $1 AND cnhm.weight IS NOT NULL
                ORDER BY case_notes.date_of_visit DESC
                {offset_and_limit};
                "
         ),
        )
        .bind(participant_id)
        .fetch_all(database)
        .await?;

        let result = PaginatedResponse {
            data,
            ..Default::default()
        };
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, Default)]
pub struct BloodPressureReadings {
    pub sit: Option<BloodPressureReading>,
    pub stand: Option<BloodPressureReading>,
    pub personal: Option<BloodPressureReading>,
}
impl From<HashMap<BloodPressureType, BloodPressureReading>> for BloodPressureReadings {
    fn from(mut map: HashMap<BloodPressureType, BloodPressureReading>) -> Self {
        BloodPressureReadings {
            sit: map.remove(&BloodPressureType::Sit),
            stand: map.remove(&BloodPressureType::Stand),
            personal: map.remove(&BloodPressureType::Personal),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(examples(BloodPressureHistory::example))]
pub struct BloodPressureHistory {
    /// Case Note It belongs to
    pub case_note_id: i32,
    /// Date of the visit
    pub date_of_visit: NaiveDate,
    /// Blood Pressure readings
    pub readings: BloodPressureReadings,
}
impl BloodPressureHistory {
    pub fn example() -> Self {
        BloodPressureHistory {
            case_note_id: 1,
            date_of_visit: NaiveDate::from_ymd_opt(2024, 9, 1).unwrap(),
            readings: BloodPressureReadings {
                sit: Some(BloodPressureReading {
                    systolic: 120,
                    diastolic: 80,
                }),
                stand: Some(BloodPressureReading {
                    systolic: 130,
                    diastolic: 90,
                }),
                personal: Some(BloodPressureReading {
                    systolic: 140,
                    diastolic: 100,
                }),
            },
        }
    }
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
        let mut readings = BloodPressureReadings::default();

        for bp in blood_pressure {
            let reading = Some(BloodPressureReading {
                systolic: bp.systolic,
                diastolic: bp.diastolic,
            });
            match bp.blood_pressure_type {
                BloodPressureType::Sit => {
                    readings.sit = reading;
                }
                BloodPressureType::Stand => {
                    readings.stand = reading;
                }
                BloodPressureType::Personal => {
                    readings.personal = reading;
                }
            }
        }

        Ok(BloodPressureHistory {
            case_note_id,
            date_of_visit,
            readings: readings,
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
    /// If page_size is 0 or less all records are returned
    pub async fn find_all_for_participant(
        participant_id: i32,
        page_and_size: impl Into<PageParams>,
        database: &sqlx::PgPool,
    ) -> DBResult<PaginatedResponse<Self>> {
        let page_and_size: PageParams = page_and_size.into();
        let offset_and_limit = if page_and_size.page_size > 0 {
            format!(
                "LIMIT {} OFFSET {}",
                page_and_size.page_size,
                page_and_size.offset()
            )
        } else {
            "".to_string()
        };

        let data = sqlx::query_as(&format!(
        "
        SELECT case_notes.id as case_note_id, case_notes.date_of_visit as date_of_visit,
            ARRAY(
                SELECT (BP.blood_pressure_type,BP.systolic, BP.diastolic) FROM health_measure_blood_pressure AS BP
                    WHERE BP.health_measure_id = HM.id
            ) as blood_pressure
            from case_note_health_measures as HM
            FULL JOIN case_notes ON case_notes.id = HM.case_note_id
            WHERE case_notes.participant_id = $1 {offset_and_limit};
            "
        ))
            .bind(participant_id).fetch_all(database).await?;

        let result = PaginatedResponse {
            data,
            ..Default::default()
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::testing::config::testing::{get_testing_config, no_testing_config};

    #[tokio::test]
    pub async fn bp_test() -> anyhow::Result<()> {
        let Some(config) = get_testing_config() else {
            no_testing_config()?;
            return Ok(());
        };
        config.init_logger();
        let database = config.database.connect().await?;
        let bps =
            super::BloodPressureHistory::find_all_for_participant(1, (0, 0), &database).await?;
        println!("{:?}", bps);
        Ok(())
    }

    #[tokio::test]
    pub async fn weight_test() -> anyhow::Result<()> {
        let Some(config) = get_testing_config() else {
            no_testing_config()?;
            return Ok(());
        };
        config.init_logger();
        let database = config.database.connect().await?;
        let bps = super::WeightHistory::find_all_for_participant(1, (0, 0), &database).await?;
        println!("{:?}", bps);
        Ok(())
    }
}
