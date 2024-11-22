use std::collections::HashMap;

use cs25_303_core::{
    database::red_cap::{
        case_notes::{new::NewBloodPressure, BloodPressureType},
        locations::Locations,
        participants::{
            goals::{NewParticipantGoal, NewParticipantGoalsSteps},
            NewDemographics, NewHealthOverview, NewMedication,
        },
    },
    red_cap::{Gender, HealthInsurance, Programs, Race, Status, VisitType},
};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use tracing::info;

use super::{RandomCompleteGoal, RandomMedication, RandomParticipant};

/// Notes we will use for data generation
///
/// This allows for the data to be consistent.
///
/// So only some people get marked as having high blood pressure
/// and some people get marked as having diabetes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParticipantExtendedInfo {
    pub has_high_blood_pressure: bool,
    pub has_diabetes: bool,
}
#[derive(Debug, Clone)]
pub struct RandomSets {
    pub rand: rand::rngs::StdRng,
    // TODO: Share a random generator
    pub participants: Vec<RandomParticipant>,
    pub goals: Vec<RandomCompleteGoal>,
    pub medications: Vec<RandomMedication>,
    pub behbehavioral_risks_identified: Vec<String>,
    pub r_locations: Vec<Locations>,
    pub m_locations: Vec<Locations>,
    pub reasons_for_visit: Vec<String>,
    pub info_provided_by_caregiver: Vec<String>,
    pub extended_patient_info: HashMap<i32, ParticipantExtendedInfo>,
}
impl Default for RandomSets {
    fn default() -> Self {
        Self {
            rand: rand::rngs::StdRng::from_entropy(),
            participants: Default::default(),
            goals: Default::default(),
            medications: Default::default(),
            behbehavioral_risks_identified: Default::default(),
            r_locations: Default::default(),
            m_locations: Default::default(),
            reasons_for_visit: Default::default(),
            info_provided_by_caregiver: Default::default(),
            extended_patient_info: Default::default(),
        }
    }
}
impl RandomSets {
    pub fn randon_behavioral_risks_identified(&mut self) -> Option<String> {
        Some(
            self.behbehavioral_risks_identified
                .choose(&mut self.rand)
                .unwrap()
                .clone(),
        )
    }
    pub fn random_health_overview(&mut self) -> NewHealthOverview {
        let height = match self.rand.gen_range(0..100) {
            0..5 => None,
            5..80 => Some(self.rand.gen_range(50..75)),
            _ => Some(self.rand.gen_range(75..84)),
        };
        let has_blood_pressure_cuff = self.rand_bool(0.5);
        let takes_more_than_5_medications = self.rand_bool(0.5);
        NewHealthOverview {
            height,
            has_blood_pressure_cuff: Some(has_blood_pressure_cuff),
            takes_more_than_5_medications: Some(takes_more_than_5_medications),
            ..Default::default()
        }
    }
    pub fn random_demographics(&mut self, gender: Gender) -> NewDemographics {
        let is_veteran = !matches!(self.rand.gen_range(0..100), 0..90);
        let (race, race_other, race_multiple) = match self.rand.gen_range(0..100) {
            0..50 => (Some(vec![Race::White]), None, None),
            50..65 => (Some(vec![Race::Black]), None, None),
            65..70 => (Some(vec![Race::Hispanic]), None, None),
            70..90 => (
                Some(vec![Race::IdentifyOther]),
                Some("Other".to_string()),
                None,
            ),
            _ => (
                Some(vec![Race::Multiracial]),
                None,
                Some("White, Black".to_string()),
            ),
        };
        let health_insurance = match self.rand.gen_range(0..100) {
            0..50 => vec![HealthInsurance::Medicaid],
            50..75 => vec![HealthInsurance::Medicare],
            75..90 => vec![HealthInsurance::Private],
            _ => vec![],
        };

        let highest_education_level = match self.rand.gen_range(0..100) {
            0..50 => None,
            50..75 => Some(cs25_303_core::red_cap::DegreeLevel::HighschoolOrGED),
            75..90 => Some(cs25_303_core::red_cap::DegreeLevel::Associates),
            _ => Some(cs25_303_core::red_cap::DegreeLevel::Bachelors),
        };
        NewDemographics {
            age: Some(self.rand.gen_range(18..85) as i16),
            gender: Some(gender),
            is_veteran: Some(is_veteran),
            race,
            race_other,
            race_multiple,
            health_insurance,
            highest_education_level,
            ..Default::default()
        }
    }
    pub fn random_medications(&mut self) -> Vec<NewMedication> {
        let number_of_meds = self.rand.gen_range(0..10);

        let mut meds: Vec<NewMedication> = Vec::with_capacity(number_of_meds);

        for _ in 0..number_of_meds {
            let random = loop {
                let random_med = self.medications.choose(&mut self.rand).unwrap();
                // Check if the medication is already in the list
                if meds.iter().any(|med| med.name == random_med.name) {
                    continue;
                }
                break random_med;
            };
            meds.push(random.create_new_medication());
        }
        meds
    }
    pub fn random_goals(&mut self) -> Vec<(NewParticipantGoal, Vec<NewParticipantGoalsSteps>)> {
        let number_of_goals = self.rand.gen_range(0..3);
        info!(?number_of_goals, "Creating goals");
        let mut goals = Vec::with_capacity(number_of_goals);
        for _ in 0..number_of_goals {
            let random = self.goals.choose(&mut self.rand).unwrap();
            goals.push(random.create_new_goal());
        }
        goals
    }
    pub fn pick_random_program(&mut self) -> Programs {
        if self.rand.gen_bool(1f64 / 3f64) {
            Programs::MHWP
        } else {
            Programs::RHWP
        }
    }
    pub fn location_for_program(&mut self, program: Programs) -> Locations {
        if program == Programs::MHWP {
            self.m_locations.choose(&mut self.rand).unwrap().clone()
        } else {
            self.r_locations.choose(&mut self.rand).unwrap().clone()
        }
    }
    pub fn random_info_by_caregiver(&mut self) -> Option<String> {
        // 50 chance of none
        if self.rand.gen_bool(0.5) {
            return None;
        }
        Some(
            self.info_provided_by_caregiver
                .choose(&mut self.rand)
                .unwrap()
                .clone(),
        )
    }
    pub fn random_reason_for_visit(&mut self) -> Option<String> {
        // 25 chance of none
        if self.rand_bool(0.25) {
            return None;
        }
        Some(
            self.reasons_for_visit
                .choose(&mut self.rand)
                .unwrap()
                .clone(),
        )
    }
    pub fn random_visit_type(&self) -> Option<VisitType> {
        match rand::thread_rng().gen_range(0..100) {
            0..10 => Some(VisitType::OnsiteAndHome),
            _ => Some(VisitType::Onsite),
        }
    }
    // TODO: Add Standing blood pressure
    pub fn random_blood_pressure(&mut self, participant: i32) -> Vec<NewBloodPressure> {
        let should_add_stand = self.rand_bool(0.25);
        if self.extended_patient_info[&participant].has_high_blood_pressure {
            let mut result = vec![NewBloodPressure {
                blood_pressure_type: BloodPressureType::Sit,
                systolic: self.rand.gen_range(130..180) as i16,
                diastolic: self.rand.gen_range(80..120) as i16,
            }];
            if should_add_stand {
                result.push(NewBloodPressure {
                    blood_pressure_type: BloodPressureType::Stand,
                    systolic: self.rand.gen_range(130..180) as i16,
                    diastolic: self.rand.gen_range(80..120) as i16,
                });
            }
            result
        } else {
            let mut result = vec![NewBloodPressure {
                blood_pressure_type: BloodPressureType::Sit,
                systolic: self.rand.gen_range(90..120) as i16,
                diastolic: self.rand.gen_range(60..80) as i16,
            }];
            if should_add_stand {
                result.push(NewBloodPressure {
                    blood_pressure_type: BloodPressureType::Stand,
                    systolic: self.rand.gen_range(90..120) as i16,
                    diastolic: self.rand.gen_range(60..80) as i16,
                });
            }
            result
        }
    }
    fn rand_bool(&mut self, chance: f64) -> bool {
        self.rand.gen_bool(chance)
    }
    pub fn create_extended_profile_for_partiicpant(
        &mut self,
        participant: i32,
    ) -> ParticipantExtendedInfo {
        // About 47% chance of having high blood pressure
        let has_high_blood_pressure = self.rand_bool(0.47);
        let has_diabetes = self.rand_bool(0.1);
        let extended = ParticipantExtendedInfo {
            has_high_blood_pressure,
            has_diabetes,
        };
        self.extended_patient_info
            .insert(participant, extended.clone());
        extended
    }
    pub fn random_phone_number(&mut self) -> String {
        let phone_number: String = format!(
            "(555) {:03}-{:04}",
            self.rand.gen_range(100..999),
            self.rand.gen_range(1000..9999)
        );
        phone_number
    }
    pub fn random_status(&mut self) -> Status {
        match self.rand.gen_range(0..100) {
            0..75 => Status::Active,
            75..85 => Status::Inactive,
            85..95 => Status::NoValidContactStatus,
            _ => Status::Deceases,
        }
    }
}
