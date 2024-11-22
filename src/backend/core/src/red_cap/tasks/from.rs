use ahash::{HashMap, HashMapExt};
use sqlx::PgPool;
use tracing::{debug, error, info, instrument};

use crate::{
    database::red_cap::{
        case_notes::{
            new::{NewCaseNote, NewCaseNoteHealthMeasures},
            CaseNote,
        },
        participants::{
            goals::{NewParticipantGoal, NewParticipantGoalsSteps},
            NewDemographics, NewHealthOverview, NewMedication, NewParticipant, ParticipantType,
            Participants,
        },
    },
    red_cap::{
        api::{ExportOptions, Forms, RedcapClient},
        converter::{
            case_notes::{
                OtherCaseNoteData, RedCapBloodPressureReadings, RedCapCaseNoteBase,
                RedCapHealthMeasures,
            },
            goals::{RedCapCompleteGoals, RedCapGoalsSteps},
            medications::RedCapMedication,
            participants::{
                RedCapHealthOverview, RedCapParticipant, RedCapParticipantDemographics,
            },
            RedCapConverter,
        },
        process_flat_json, RedCapExportDataType,
    },
};

use super::RedCapTaskError;

pub async fn pull_record_base_types(
    record_id: i32,
    database: &PgPool,
    converter: &mut RedCapConverter,
    client: &RedcapClient,
) -> Result<(), RedCapTaskError> {
    let mut records = client
        .get_flat_json_forms(ExportOptions {
            forms: Some(vec![Forms::ParticipantInformation, Forms::HealthOverview].into()),
            records: Some(vec![record_id as usize].into()),

            ..Default::default()
        })
        .await?;
    let first_record = records.remove(0);
    let record = process_flat_json(first_record);

    let red_cap_participant = RedCapParticipant::read_participant(&record, converter).await?;
    let demographics = RedCapParticipantDemographics::read(&record).await?;
    let overview = RedCapHealthOverview::read(&record).await?;
    debug!(
        ?red_cap_participant,
        ?demographics,
        ?overview,
        "Read red cap data"
    );
    if let Some(mut participant) = Participants::find_by_red_cap_id(record_id, database).await? {
        info!(?participant, "Updating participant from red cap");
        participant
            .update_from_red_cap(red_cap_participant, demographics, overview, database)
            .await?;
    } else {
        let new_participant: NewParticipant = red_cap_participant.into();
        let new_demographics: Option<NewDemographics> = demographics.into();
        let new_overview: NewHealthOverview = overview.into();

        let participant = new_participant.insert_return_participant(database).await?;

        if let Some(demographics) = new_demographics {
            demographics.insert_none(participant.id, database).await?;
        }

        new_overview
            .insert_return_none(participant.id, database)
            .await?;
    }
    Ok(())
}
#[instrument]
pub async fn pull_medications(
    record_id: i32,
    database: &PgPool,
    client: &RedcapClient,
) -> Result<(), RedCapTaskError> {
    let Some(participant) = Participants::find_by_red_cap_id(record_id, database).await? else {
        error!(?record_id, "Participant must be loaded before medications");
        return Err(RedCapTaskError::ParticipantNotFound);
    };
    let mut records = client
        .get_flat_json_forms(ExportOptions {
            forms: Some(vec![Forms::Medications].into()),
            records: Some(vec![record_id as usize].into()),
            ..Default::default()
        })
        .await?;
    let first_record = records.remove(0);
    let record = process_flat_json(first_record);
    let medications = RedCapMedication::read(&record);
    for medication in medications {
        debug!(?medication, ?participant, "Read medication from red cap");
        let new_medication: NewMedication = medication.into();

        new_medication
            .insert_return_none(participant.id, database)
            .await?;
    }
    Ok(())
}
#[instrument]
pub async fn pull_goals(
    record_id: i32,
    database: &PgPool,
    client: &RedcapClient,
) -> Result<(), RedCapTaskError> {
    let Some(participant) = Participants::find_by_red_cap_id(record_id, database).await? else {
        error!(?record_id, "Participant must be loaded before goals");
        return Err(RedCapTaskError::ParticipantNotFound);
    };
    let mut records = client
        .get_flat_json_forms(ExportOptions {
            forms: Some(vec![Forms::WellnessGoals].into()),
            records: Some(vec![record_id as usize].into()),
            ..Default::default()
        })
        .await?;
    let first_record = records.remove(0);
    let record = process_flat_json(first_record);
    let RedCapCompleteGoals { goals, steps } = RedCapCompleteGoals::read(&record)?;
    // TODO: Mark Old Goals as hidden_from_red_cap
    // TODO: Try to match goals to existing goals based on red_cap_index

    let mut index_to_goal = HashMap::new();

    for goal in goals {
        debug!(?goal, ?participant, "Read medication from red cap");
        let index = goal.red_cap_index;
        let new_goal: NewParticipantGoal = goal.into();

        let goal = new_goal
            .insert_return_goal(participant.id, database)
            .await?;

        index_to_goal.insert(index, goal);
    }

    for step in steps {
        debug!(?step, ?participant, "Read medication from red cap");
        let RedCapGoalsSteps {
            associated_goal,
            step,
            confidence_level,
            date_set,
            date_to_be_completed,
            action_step,
            red_cap_index,
        } = step;
        let goal_id = if let Some(goal_id) = associated_goal {
            let goal = index_to_goal.get(&goal_id);
            debug!(?goal, "Goal found");
            goal.map(|x| x.id)
        } else {
            None
        };
        let new_step: NewParticipantGoalsSteps = NewParticipantGoalsSteps {
            goal_id,
            step,
            confidence_level,
            date_set,
            date_to_be_completed,
            action_step,
            red_cap_index: Some(red_cap_index),
        };

        new_step
            .insert_return_none(participant.id, database)
            .await?;
    }
    Ok(())
}
pub async fn pull_case_notes(
    record_id: i32,
    database: &PgPool,
    converter: &mut RedCapConverter,
    client: &RedcapClient,
) -> Result<(), RedCapTaskError> {
    let Some(participant) = Participants::find_by_red_cap_id(record_id, database).await? else {
        error!(?record_id, "Participant must be loaded before goals");
        return Err(RedCapTaskError::ParticipantNotFound);
    };
    let records = client
        .get_flat_json_forms(ExportOptions {
            forms: Some(vec![Forms::CaseNotes].into()),
            records: Some(vec![record_id as usize].into()),
            fields: Some(vec!["record_id".to_owned()].into()),
        })
        .await?;

    for case_note in records {
        let record = process_flat_json(case_note);

        add_case_note(&participant, record, database, converter).await?;
    }

    Ok(())
}

async fn add_case_note(
    participants: &Participants,
    record: HashMap<String, RedCapExportDataType>,
    database: &PgPool,
    converter: &mut RedCapConverter,
) -> Result<(), RedCapTaskError> {
    let Some(case_note) = RedCapCaseNoteBase::read_case_note_base(&record, converter).await? else {
        // Case Note does not have an instance number. Meaning it isn't complete?
        return Ok(());
    };

    let health_measures = RedCapHealthMeasures::read_health_measures(&record).await?;

    let bp_readings = RedCapBloodPressureReadings::read(&record);

    let other = OtherCaseNoteData::read(&record, converter).await?;
    if let Some(existing_case_note) = CaseNote::find_by_participant_id_and_redcap_instance(
        participants.id,
        case_note.red_cap_instance.unwrap(),
        database,
    )
    .await?
    {
        // Update the case note
        existing_case_note
            .update_from_red_cap(case_note, health_measures, other, database)
            .await?;
    } else {
        // Insert the case note
        let new_case_note: NewCaseNote = case_note.into();
        let new_health_measures: NewCaseNoteHealthMeasures = health_measures.into();

        let case_note = new_case_note
            .insert_return_case_note(participants.id, database)
            .await?;

        let measures = new_health_measures
            .insert_return_measure(case_note.id, database)
            .await?;
        // TODO: Use 1 INSERT statement for all the blood pressures
        for bp in bp_readings.readings {
            measures.add_bp(bp, database).await?;
        }
        for (question_id, value) in other.values {
            debug!(?question_id, ?value, "Adding question");
            crate::database::red_cap::case_notes::questions::add_question(
                question_id,
                case_note.id,
                value,
                database,
            )
            .await?;
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use tracing::info;

    use crate::red_cap::{converter::RedCapConverter, tests::load_red_cap_api_and_db};

    #[tokio::test]
    #[ignore]
    pub async fn import_base_info_for_record_one() -> anyhow::Result<()> {
        let (client, database) = load_red_cap_api_and_db().await?;

        let mut converter: RedCapConverter = RedCapConverter::new(database.clone()).await?;
        info!("Pulling record 1");
        super::pull_record_base_types(1, &database, &mut converter, &client).await?;
        info!("Pulling Medications for record 1");
        super::pull_medications(1, &database, &client).await?;
        info!("Pulling Goals for record 1");
        super::pull_goals(1, &database, &client).await?;
        info!("Pulling Case Notes for record 1");
        super::pull_case_notes(1, &database, &mut converter, &client).await?;
        Ok(())
    }
}
