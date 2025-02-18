use std::fmt::Debug;

use crate::database::{prelude::*, PaginatedResponse};
use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use tracing::{debug, instrument, trace, warn};
use utoipa::ToSchema;

use crate::red_cap::MedicationFrequency;

use super::TableType;
/// Participant Medications
///
/// Table Name: participant_medications
///
/// Relationships:
/// * Belongs to [Participants](crate::database::red_cap::participants::Participants)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow, TableType, ToSchema)]
#[table(name = "participant_medications")]
pub struct ParticipantMedications {
    /// The ID of the medication
    pub id: i32,
    /// The ID of the participant
    pub participant_id: i32,
    /// The name of the medication
    pub name: String,
    /// The dosage of the medication
    pub dosage: Option<String>,
    /// The frequency of the medication
    pub frequency: Option<MedicationFrequency>,
    /// The date the medication was prescribed
    ///
    /// Null if not known
    pub date_prescribed: Option<NaiveDate>,
    /// The date the medication was entered into the system
    ///
    /// Defaults to the current date
    pub date_entered_into_system: Option<NaiveDate>,
    pub is_current: Option<bool>,

    pub date_discontinued: Option<NaiveDate>,
    /// Comments about the medication
    pub comments: Option<String>,
    /// The index of the medication in red cap
    pub red_cap_index: Option<i32>,
    /// Whether the medication is hidden from red cap
    ///
    /// This is done when we hit past the 40 medication limit
    pub hidden_from_red_cap: bool,
    /// When the medication was inserted into the database
    pub created_at: chrono::DateTime<FixedOffset>,
}
impl ParticipantMedications {
    pub async fn get_all_participant_medications(
        participant_id: i32,
        database: &PgPool,
    ) -> DBResult<Vec<ParticipantMedications>> {
        let result = SelectQueryBuilder::with_columns(
            ParticipantMedications::table_name(),
            ParticipantMedicationsColumn::all(),
        )
        .filter(ParticipantMedicationsColumn::ParticipantId.equals(participant_id.value()))
        .query_as()
        .fetch_all(database)
        .await?;
        Ok(result)
    }
    /// A Paginated Search for medications
    #[instrument]
    pub async fn search_medications(
        participant_id: i32,
        database: &PgPool,
        name: Option<String>,
        params: PageParams,
    ) -> DBResult<PaginatedResponse<ParticipantMedications>> {
        let name = name
            .as_ref()
            .map(|name| format!("%{}%", name.to_lowercase()));
        trace!(?name, ?participant_id, ?params, "Searching for medications");
        let count = {
            let mut query = SelectCount::new(ParticipantMedications::table_name());
            query.filter(ParticipantMedicationsColumn::ParticipantId.equals(participant_id));
            if let Some(name) = &name {
                query.filter(
                    ParticipantMedicationsColumn::Name
                        .lower()
                        .like(name.as_str()),
                );
            }
            query.execute(database).await?
        };
        if count == 0 {
            debug!(?name, ?params, "No medications found");
            return Ok(PaginatedResponse::default());
        }

        if count < params.offset() as i64 {
            debug!(
                ?name,
                ?params,
                ?count,
                "The offset os greater than the count"
            );
            return Ok(PaginatedResponse::default());
        }

        let mut query = SelectQueryBuilder::with_columns(
            ParticipantMedications::table_name(),
            ParticipantMedicationsColumn::all(),
        );
        query.filter(ParticipantMedicationsColumn::ParticipantId.equals(participant_id));
        if let Some(name) = name {
            query.filter(ParticipantMedicationsColumn::Name.lower().like(name));
        }
        query.page_params(params);
        let result = query.query_as().fetch_all(database).await?;

        let result = PaginatedResponse {
            total_pages: params.number_of_pages(count),
            total: count,
            data: result,
        };
        Ok(result)
    }
    /// Returns the number of medications for a participant with a name search
    #[tracing::instrument]
    pub async fn count_medications_with_name_search(
        participant_id: i32,
        database: &PgPool,
        name: Option<&str>,
    ) -> DBResult<i64> {
        let mut query = SelectCount::new(ParticipantMedications::table_name());
        query.filter(ParticipantMedicationsColumn::ParticipantId.equals(participant_id));
        if let Some(name) = name {
            query.filter(
                ParticipantMedicationsColumn::Name
                    .lower()
                    .like(format!("%{}%", name.to_lowercase())),
            );
        }
        Ok(query.execute(database).await?)
    }
    /// Returns the number of medications for a participant
    #[tracing::instrument]
    pub async fn count_medications_for_participant(
        participant_id: i32,
        database: &PgPool,
    ) -> DBResult<i64> {
        let query = SelectCount::new(ParticipantMedications::table_name())
            .filter(ParticipantMedicationsColumn::ParticipantId.equals(participant_id))
            .execute(database)
            .await?;

        Ok(query)
    }

    /// Will ensure each medication in has a red_cap_index.
    ///
    /// This will also make sure no "gaps" exist in the red_cap_index.
    pub async fn process_medications_indexes(
        participant_id: i32,
        database: &PgPool,
    ) -> DBResult<()> {
        let mut medications =
            Self::get_all_participant_medications(participant_id, database).await?;
        medications.sort_by(|a, b| {
            a.red_cap_index
                .unwrap_or(i32::MAX)
                .cmp(&b.red_cap_index.unwrap_or(i32::MAX))
        });

        for (index, medication) in medications.iter_mut().enumerate() {
            let red_cap_index = index as i32 + 1;
            medication
                .set_red_cap_index(red_cap_index, database)
                .await?;
        }
        Ok(())
    }

    pub async fn set_red_cap_index(
        &mut self,
        red_cap_index: i32,
        database: &PgPool,
    ) -> DBResult<()> {
        if self.red_cap_index == Some(red_cap_index) {
            return Ok(());
        }
        self.red_cap_index = Some(red_cap_index);

        sqlx::query("UPDATE participant_medications SET red_cap_index = $1 WHERE id = $2")
            .bind(red_cap_index)
            .bind(self.id)
            .execute(database)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct NewMedication {
    pub name: String,
    pub dosage: Option<String>,
    pub frequency: Option<MedicationFrequency>,
    pub date_prescribed: Option<NaiveDate>,
    pub date_entered_into_system: Option<NaiveDate>,
    pub is_current: Option<bool>,
    pub date_discontinued: Option<NaiveDate>,
    pub comments: Option<String>,
    pub red_cap_index: Option<i32>,
}
impl NewMedication {
    pub async fn insert_return_none(
        self,
        participant_id: i32,
        database: impl Executor<'_, Database = sqlx::Postgres>,
    ) -> DBResult<()> {
        let Self {
            name,
            dosage,
            frequency,
            date_prescribed,
            date_entered_into_system,
            is_current,
            date_discontinued,
            comments,
            red_cap_index,
        } = self;

        let date_entered_into_system =
            date_entered_into_system.unwrap_or_else(|| Local::now().date_naive());
        InsertQueryBuilder::new(ParticipantMedications::table_name())
            .insert(
                ParticipantMedicationsColumn::ParticipantId,
                participant_id.value(),
            )
            .insert(ParticipantMedicationsColumn::Name, name.value())
            .insert(ParticipantMedicationsColumn::Dosage, dosage.value())
            .insert(ParticipantMedicationsColumn::Frequency, frequency.value())
            .insert(
                ParticipantMedicationsColumn::DatePrescribed,
                date_prescribed.value(),
            )
            .insert(
                ParticipantMedicationsColumn::DateEnteredIntoSystem,
                date_entered_into_system.value(),
            )
            .insert(ParticipantMedicationsColumn::IsCurrent, is_current.value())
            .insert(
                ParticipantMedicationsColumn::DateDiscontinued,
                date_discontinued.value(),
            )
            .insert(ParticipantMedicationsColumn::Comments, comments.value())
            .insert(
                ParticipantMedicationsColumn::RedCapIndex,
                red_cap_index.value(),
            )
            .query()
            .execute(database)
            .await?;
        Ok(())
    }
    #[instrument]
    pub async fn insert_many(
        medications: Vec<NewMedication>,
        participant_id: i32,
        database: &PgPool,
    ) -> DBResult<()> {
        if medications.is_empty() {
            warn!("No medications to insert");
            return Ok(());
        }
        let mut query_builder = InsertManyBuilder::new(
            ParticipantMedications::table_name(),
            vec![
                ParticipantMedicationsColumn::ParticipantId,
                ParticipantMedicationsColumn::Name,
                ParticipantMedicationsColumn::Dosage,
                ParticipantMedicationsColumn::Frequency,
                ParticipantMedicationsColumn::DatePrescribed,
                ParticipantMedicationsColumn::DateEnteredIntoSystem,
                ParticipantMedicationsColumn::IsCurrent,
                ParticipantMedicationsColumn::DateDiscontinued,
                ParticipantMedicationsColumn::Comments,
                ParticipantMedicationsColumn::RedCapIndex,
            ],
        );
        for medication in medications {
            let Self {
                name,
                dosage,
                frequency,
                date_prescribed,
                date_entered_into_system,
                is_current,
                date_discontinued,
                comments,
                red_cap_index,
            } = medication;

            let date_entered_into_system =
                date_entered_into_system.unwrap_or_else(|| Local::now().date_naive());

            query_builder.insert_row_ordered(|row| {
                row.insert(participant_id.value())
                    .insert(name.value())
                    .insert(dosage.value())
                    .insert(frequency.value())
                    .insert(date_prescribed.value())
                    .insert(date_entered_into_system.value())
                    .insert(is_current.value())
                    .insert(date_discontinued.value())
                    .insert(comments.value())
                    .insert(red_cap_index.value());
            });
        }
        query_builder.query().execute(database).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::{
        database::{
            red_cap::participants::{NewMedication, NewParticipant, ParticipantMedications},
            DBError,
        },
        utils::testing::config::testing::{get_testing_db, no_db_connection},
    };

    async fn new_participant(db: &PgPool) -> Result<i32, DBError> {
        let participant = NewParticipant {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            other_contact: Some("New Medictions Test".to_owned()),
            ..Default::default()
        };

        let part = participant.insert_returning(db).await?;
        Ok(part.id)
    }

    #[tokio::test]
    async fn test_insert_new_medication() -> anyhow::Result<()> {
        let Some(database) = get_testing_db().await else {
            no_db_connection()?;
            return Ok(());
        };
        let participant_id = new_participant(&database).await?;
        let medications = vec![
            NewMedication {
                name: "Medication 1".to_string(),
                ..Default::default()
            },
            NewMedication {
                name: "Medication 2".to_string(),
                ..Default::default()
            },
            NewMedication {
                name: "Medication 3".to_string(),
                ..Default::default()
            },
        ];
        NewMedication::insert_many(medications, participant_id, &database).await?;

        let count =
            ParticipantMedications::count_medications_for_participant(participant_id, &database)
                .await?;
        assert_eq!(count, 3);

        let medications =
            ParticipantMedications::get_all_participant_medications(participant_id, &database)
                .await?;

        assert_eq!(medications.len(), 3);
        assert_eq!(medications[0].name, "Medication 1");
        assert_eq!(medications[1].name, "Medication 2");
        assert_eq!(medications[2].name, "Medication 3");

        Ok(())
    }
}
