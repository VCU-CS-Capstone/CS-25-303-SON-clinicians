use std::path::Path;
mod core;
mod set;
use chrono::Local;
pub use core::*;
use cs25_303_core::{
    database::red_cap::{
        case_notes::new::{NewCaseNote, NewCaseNoteHealthMeasures},
        locations::Locations,
        participants::{
            goals::{ParticipantGoals, ParticipantGoalsSteps},
            NewParticipant, ParticipantMedications, Participants,
        },
    },
    red_cap::Programs,
};
use rand::{seq::SliceRandom, Rng};
use serde::de::DeserializeOwned;
use set::RandomSets;
use sqlx::{types::chrono::NaiveDate, PgPool};
use tracing::info;

fn load_random_set<T>(name: &str) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("random")
        .join("sets")
        .join(format!("{}.json", name));
    info!("Loading random set from path: {:?}", path);
    let file = std::fs::read_to_string(path)?;
    let random_sets: T = serde_json::from_str(&file)?;
    Ok(random_sets)
}
pub async fn load_random_sets(database: Option<&PgPool>) -> anyhow::Result<RandomSets> {
    let (r_locations, m_locations) = if let Some(database) = database {
        (
            Locations::find_all_in_program(Programs::RHWP, database).await?,
            Locations::find_all_in_program(Programs::MHWP, database).await?,
        )
    } else {
        (vec![], vec![])
    };

    Ok(RandomSets {
        participants: load_random_set("participants")?,
        goals: load_random_set("goals")?,
        medications: load_random_set("medications")?,
        behbehavioral_risks_identified: load_random_set("behavioral_risks_identified")?,
        r_locations,
        m_locations,
        reasons_for_visit: load_random_set("reason_for_visit")?,
        info_provided_by_caregiver: load_random_set("info_by_caregiver")?,
        ..Default::default()
    })
}

pub async fn generate_participants(count: usize, database: PgPool) -> anyhow::Result<()> {
    let mut random_sets = load_random_sets(Some(&database)).await?;

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
        let number_of_case_notes = random_sets.rand.gen_range(0..10);
        let signed_up_on =
            Local::now().date_naive() - chrono::Duration::weeks(number_of_case_notes);

        let new_participant = NewParticipant {
            first_name,
            last_name,
            red_cap_id: None,
            phone_number_one: Some(random_sets.random_phone_number()),
            phone_number_two: None,
            other_contact: Some("Randomly Generated Participant. By Wyatt Herkamp".to_string()),
            program: program_and_location,
            location: Some(location.id),
            status: Some(random_sets.random_status()),
            behavioral_risks_identified: random_sets.randon_behavioral_risks_identified(),
            date_care_coordination_consent_signed: None,
            date_home_visit_consent_signed: None,
            signed_up_on,
            last_synced_with_redcap: None,
        };
        let part = new_participant.insert_return_participant(&database).await?;
        let extra_info = random_sets.create_extended_profile_for_partiicpant(part.id);
        info!("Created Participant {:?} and extra {:?}", part, extra_info);
        let health_overview = random_sets.random_health_overview();
        health_overview
            .insert_return_none(part.id, &database)
            .await?;

        let demographics = random_sets.random_demographics(gender);

        demographics.insert_none(part.id, &database).await?;

        let medications = random_sets.random_medications();

        for medication in medications {
            medication.insert_return_none(part.id, &database).await?;
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
    let visit_type = random.random_visit_type();
    let reason_for_visit = random.random_reason_for_visit();
    let info_provided_by_caregiver = random.random_info_by_caregiver();

    let new_case_note = NewCaseNote {
        location: participant.location,
        visit_type,
        age: None, // TODO: Pass Age Into this function
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
    let new_health_measures = NewCaseNoteHealthMeasures::default();
    let health_measure = new_health_measures
        .insert_return_measure(case_note.id, database)
        .await?;
    for bp in bps {
        health_measure.add_bp(bp, database).await?;
    }

    Ok(())
}
#[cfg(test)]

mod tests {

    #[tokio::test]
    pub async fn load_full() -> anyhow::Result<()> {
        let random_sets = super::load_random_sets(None).await?;
        println!("{:#?}", random_sets);
        Ok(())
    }
}
