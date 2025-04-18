pub mod blood_pressure;
pub mod other;
pub use blood_pressure::*;
pub use other::*;
#[cfg(test)]
mod tests {
    use chrono::{Duration, Local};
    use rand::Rng;

    use crate::{
        database::{
            CSPageParams, DBError,
            red_cap::{
                case_notes::{
                    BloodPressureType,
                    new::{NewBloodPressure, NewCaseNote, NewCaseNoteHealthMeasures},
                },
                participants::NewParticipant,
            },
        },
        utils::testing::config::testing::{get_testing_db, no_db_connection},
    };
    async fn create_participant_with_history(
        database: &sqlx::PgPool,
        connect_message: &str,
    ) -> Result<i32, DBError> {
        let mut random = rand::rng();
        let participant = NewParticipant {
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            other_contact: Some(connect_message.to_string()),
            ..NewParticipant::default()
        }
        .insert_returning(database)
        .await?;
        for i in 0..15 {
            let case_note = NewCaseNote {
                location: Some(1),
                date_of_visit: Local::now().date_naive() - Duration::weeks(16 - i),
                ..NewCaseNote::default()
            }
            .insert_return_case_note(participant.id, database)
            .await?;
            let health_measure = NewCaseNoteHealthMeasures {
                weight: Some(random.random_range(180f32..190f32)),
                glucose_result: Some(random.random_range(100f32..110f32)),
                fasted_atleast_2_hours: Some(random.random_bool(0.50)),
                ..Default::default()
            };

            let measure = health_measure
                .insert_return_measure(case_note.id, database)
                .await?;

            let bps = vec![
                NewBloodPressure {
                    blood_pressure_type: BloodPressureType::Sit,
                    systolic: random.random_range(120..130),
                    diastolic: random.random_range(80..90),
                },
                NewBloodPressure {
                    blood_pressure_type: BloodPressureType::Stand,
                    systolic: random.random_range(130..140),
                    diastolic: random.random_range(90..100),
                },
                NewBloodPressure {
                    blood_pressure_type: BloodPressureType::Personal,
                    systolic: random.random_range(140..150),
                    diastolic: random.random_range(100..110),
                },
            ];
            measure.add_many_bp(bps, database).await?;
        }

        Ok(participant.id)
    }
    #[tokio::test]
    pub async fn bp_test() -> anyhow::Result<()> {
        let Some(database) = get_testing_db().await else {
            no_db_connection()?;
            return Ok(());
        };
        let participant_id =
            create_participant_with_history(&database, "CS25-303 bp_tests").await?;

        let bps = super::BloodPressureHistory::find_all_for_participant(
            participant_id,
            CSPageParams {
                page_number: 1,
                page_size: 15,
            },
            &database,
        )
        .await?;
        assert_eq!(bps.data.len(), 15);
        Ok(())
    }

    #[tokio::test]
    pub async fn weight_test() -> anyhow::Result<()> {
        let Some(database) = get_testing_db().await else {
            no_db_connection()?;
            return Ok(());
        };
        let participant_id =
            create_participant_with_history(&database, "CS25-303 weight_test").await?;
        let weights = super::WeightHistory::find_all_for_participant(
            participant_id,
            true,
            CSPageParams {
                page_number: 1,
                page_size: 15,
            },
            &database,
        )
        .await?;

        assert_eq!(weights.data.len(), 15);

        println!("{:?}", weights);
        Ok(())
    }
    #[tokio::test]
    pub async fn glucose_test() -> anyhow::Result<()> {
        let Some(database) = get_testing_db().await else {
            no_db_connection()?;
            return Ok(());
        };
        let participant_id =
            create_participant_with_history(&database, "CS25-303 glucose test").await?;
        let weights = super::BloodGlucoseHistory::find_all_for_participant(
            participant_id,
            CSPageParams {
                page_number: 1,
                page_size: 15,
            },
            &database,
        )
        .await?;

        assert_eq!(weights.data.len(), 15);

        println!("{:?}", weights);
        Ok(())
    }
}
