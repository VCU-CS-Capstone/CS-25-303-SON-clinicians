use crate::database::{
    PaginatedResponse,
    prelude::*,
    red_cap::case_notes::{
        BloodPressureType, CaseNote, CaseNoteColumn, CaseNoteHealthMeasures,
        CaseNoteHealthMeasuresColumn, HealthMeasureBloodPressure, HealthMeasureBloodPressureColumn,
    },
};
use ahash::HashMap;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
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
            readings,
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
        let mut query = SelectQueryBuilder::new(CaseNoteHealthMeasures::table_name());
        query
            .select(CaseNoteColumn::Id.alias("case_note_id"))
            .select(CaseNoteColumn::DateOfVisit.alias("date_of_visit"))
            .select(
                SqlFunctionBuilder::array()
                    .add_param(
                        SelectExprBuilder::new(HealthMeasureBloodPressure::table_name())
                            .column(HealthMeasureBloodPressureColumn::BloodPressureType)
                            .column(HealthMeasureBloodPressureColumn::Systolic)
                            .column(HealthMeasureBloodPressureColumn::Diastolic)
                            .filter(
                                HealthMeasureBloodPressureColumn::HealthMeasureId
                                    .equals(CaseNoteHealthMeasuresColumn::Id),
                            ),
                    )
                    .alias("blood_pressure"),
            )
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

        let result: PaginatedResponse<BloodPressureHistory> = PaginatedResponse::from_rows(
            query.query().fetch_all(database).await?,
            &page_and_size,
            "total_entries",
        )?;
        Ok(result)
    }
}
