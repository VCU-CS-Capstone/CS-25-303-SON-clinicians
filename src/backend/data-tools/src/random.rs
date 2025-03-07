mod core;
mod set;
use chrono::Local;
use clap::Args;
pub use core::*;
use cs25_303_core::{
    database::red_cap::{
        case_notes::new::{NewCaseNote, NewCaseNoteHealthMeasures},
        participants::{
            NewMedication, NewParticipant, ParticipantMedications, Participants,
            goals::{ParticipantGoals, ParticipantGoalsSteps},
        },
    },
    red_cap::SeenAtVCUHS,
};
use data::load_random_sets;
pub mod data;
pub mod utils;
use rand::{Rng, seq::IndexedRandom};
use set::RandomSets;
use sqlx::{PgPool, types::chrono::NaiveDate};
use tracing::info;

use crate::config::DataToolConfig;
#[derive(Debug, Clone, Args)]
pub struct RandomParticipantsCommand {
    #[clap(short, long, default_value = "50")]
    pub count: usize,
}
impl RandomParticipantsCommand {
    pub async fn run(self, config: DataToolConfig) -> anyhow::Result<()> {
        let database = cs25_303_core::database::connect(config.database.try_into()?, true).await?;

        println!("Generating {} participants", self.count);
        generate_participants(self.count, database).await
    }
}

pub async fn generate_participants(count: usize, database: PgPool) -> anyhow::Result<()> {
    let mut random_sets = load_random_sets(None)?;
    random_sets.load_locations(&database).await?;
    for _ in 0..count {
        let RandomParticipant {
            first_name,
            last_name,
            gender,
        } = random_sets
            .participants
            .choose(&mut random_sets.rand)
            .unwrap()
            .clone();
        let program_and_location = random_sets.pick_random_program();
        let location = random_sets.location_for_program(program_and_location);
        let (signed_up_on, number_of_case_notes) = random_sets.first_week_and_numer_of_case_notes();
        let vcuhs = match random_sets.rand.random_range(0..100) {
            0..50 => Some(SeenAtVCUHS::Yes),
            50..75 => Some(SeenAtVCUHS::No),
            75..85 => Some(SeenAtVCUHS::Unsure),
            85..95 => Some(SeenAtVCUHS::DidNotAsk),
            _ => None,
        };
        let new_participant = NewParticipant {
            first_name,
            last_name,
            red_cap_id: None,
            phone_number_one: Some(random_sets.random_phone_number()),
            phone_number_two: None,
            // This string is intentionally this message as it will be used to identify these random participants
            other_contact: Some("Randomly Generated Participant. By Wyatt Herkamp".to_string()),
            program: program_and_location,
            vcuhs_patient_status: vcuhs,
            location: Some(location.id),
            status: Some(random_sets.random_status()),
            behavioral_risks_identified: random_sets.random_behavioral_risks_identified(),
            date_care_coordination_consent_signed: None,
            date_home_visit_consent_signed: None,
            signed_up_on,
            last_synced_with_redcap: None,
        };
        let part = new_participant.insert_returning(&database).await?;
        let extra_info =
            random_sets.create_extended_profile_for_partiicpant(part.id, gender.clone());
        info!("Created Participant {:?} and extra {:?}", part, extra_info);
        let health_overview = random_sets.random_health_overview();
        health_overview.insert(part.id, &database).await?;

        let demographics = random_sets.random_demographics(part.id);

        demographics.insert(part.id, &database).await?;
        let medications = random_sets.random_medications();
        if !medications.is_empty() {
            NewMedication::insert_many(medications, part.id, &database).await?;
        }
        let goals = random_sets.random_goals();

        for (goal, steps) in goals {
            let goal = goal.insert_return_goal(part.id, &database).await?;
            for step in steps {
                step.insert_with_goal_return_none(part.id, goal.id, &database)
                    .await?;
            }
        }
        let current_date = Local::now().date_naive();
        for i in 0..number_of_case_notes {
            let date_of_visit = current_date - chrono::Duration::weeks(i);
            info!("Creating Case Note for {:?} on {:?}", part, date_of_visit);
            generate_random_case_note_on(&mut random_sets, part.clone(), date_of_visit, &database)
                .await?;
        }
        ParticipantGoals::process_red_cap_indexes(part.id, &database).await?;
        ParticipantGoalsSteps::process_red_cap_indexes(part.id, &database).await?;
        ParticipantMedications::process_medications_indexes(part.id, &database).await?;
    }
    Ok(())
}

async fn generate_random_case_note_on(
    random: &mut RandomSets,
    participant: Participants,
    date_of_visit: NaiveDate,
    database: &PgPool,
) -> anyhow::Result<()> {
    let age = random
        .extended_patient_info
        .get(&participant.id)
        .unwrap()
        .age;
    let visit_type = random.random_visit_type();
    let reason_for_visit = random.random_reason_for_visit();
    let info_provided_by_caregiver = random.random_info_by_caregiver();

    let new_case_note = NewCaseNote {
        location: participant.location,
        visit_type,
        age: Some(age),
        reason_for_visit,
        info_provided_by_caregiver,
        date_of_visit,
        completed: true,
        ..Default::default()
    };
    let case_note = new_case_note
        .insert_return_case_note(participant.id, database)
        .await?;
    let bps = random.random_blood_pressure(participant.id);
    let (glucose_tested, glucose_value, fasted_2_hours) =
        random.random_glucose_test(participant.id);
    let new_health_measures = NewCaseNoteHealthMeasures {
        weight: random.weight_for_participant(participant.id),
        glucose_tested,
        glucose_result: glucose_value,
        fasted_atleast_2_hours: fasted_2_hours,
        other: None,
    };
    let health_measure = new_health_measures
        .insert_return_measure(case_note.id, database)
        .await?;
    health_measure.add_many_bp(bps, database).await?;
    // TODO: Adding the questions

    Ok(())
}
#[cfg(test)]
mod tests {}
