use ahash::HashMap;
use chrono::{Local, NaiveDate};
use cs25_303_core::{
    database::red_cap::{
        case_notes::{BloodPressureType, new::NewBloodPressure},
        locations::Locations,
        participants::{
            NewDemographics, NewHealthOverview, NewMedication,
            goals::{NewParticipantGoal, NewParticipantGoalsSteps},
        },
    },
    red_cap::{
        Ethnicity, Gender, HealthInsurance, PreferredLanguage, Programs, Race, Status, VisitType,
    },
};
use rand::{Rng, SeedableRng, seq::IndexedRandom};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{info, warn};

use super::{
    RandomCompleteGoal, RandomMedication, random_user::RandomUserAPIClient, utils::RandDate,
};
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub enum WeightCategory {
    Underweight,
    Overweight,
    #[default]
    Normal,
}
impl WeightCategory {
    pub fn weight_for_female_in_class(&self, random: &mut impl Rng) -> f32 {
        match self {
            WeightCategory::Underweight => random.random_range(90..120) as f32,
            WeightCategory::Overweight => random.random_range(160..200) as f32,
            WeightCategory::Normal => random.random_range(120..160) as f32,
        }
    }
    pub fn weight_for_male_in_class(&self, random: &mut impl Rng) -> f32 {
        match self {
            WeightCategory::Underweight => random.random_range(100..140) as f32,
            WeightCategory::Overweight => random.random_range(180..220) as f32,
            WeightCategory::Normal => random.random_range(140..180) as f32,
        }
    }
    pub fn generic_weight_class(&self, random: &mut impl Rng) -> f32 {
        match self {
            WeightCategory::Underweight => random.random_range(90..140) as f32,
            WeightCategory::Overweight => random.random_range(160..220) as f32,
            WeightCategory::Normal => random.random_range(120..180) as f32,
        }
    }
}
/// Notes we will use for data generation
///
/// This allows for the data to be consistent.
///
/// So only some people get marked as having high blood pressure
/// and some people get marked as having diabetes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParticipantExtendedInfo {
    /// If the participant has high blood pressure
    pub has_high_blood_pressure: bool,
    /// If the participant has a blood pressure cuff
    pub has_blood_pressure_cuff: bool,
    /// If the participant has diabetes
    pub has_diabetes: bool,
    /// The age of the participant
    pub age: i16,
    pub gender: Gender,
    pub weight_category: WeightCategory,
}

impl ParticipantExtendedInfo {
    pub fn weight_entry(&self, random: &mut impl Rng) -> f32 {
        match self.gender {
            Gender::Female => self.weight_category.weight_for_female_in_class(random),
            Gender::Male => self.weight_category.weight_for_male_in_class(random),
            _ => self.weight_category.generic_weight_class(random),
        }
    }
}
#[derive(Debug, Clone)]
pub struct RandomSets {
    pub rand: rand::rngs::StdRng,
    pub goals: Vec<RandomCompleteGoal>,
    pub medications: Vec<RandomMedication>,
    pub behbehavioral_risks_identified: Vec<String>,
    pub r_locations: Vec<Locations>,
    pub m_locations: Vec<Locations>,
    pub reasons_for_visit: Vec<String>,
    pub info_provided_by_caregiver: Vec<String>,
    pub case_note_other_health_measures: Vec<String>,
    pub extended_patient_info: HashMap<i32, ParticipantExtendedInfo>,
    pub random_user_client: RandomUserAPIClient,
}
impl Default for RandomSets {
    fn default() -> Self {
        Self {
            rand: rand::rngs::StdRng::from_os_rng(),
            random_user_client: RandomUserAPIClient::default(),
            goals: Default::default(),
            medications: Default::default(),
            behbehavioral_risks_identified: Default::default(),
            r_locations: Default::default(),
            m_locations: Default::default(),
            reasons_for_visit: Default::default(),
            info_provided_by_caregiver: Default::default(),
            extended_patient_info: Default::default(),
            case_note_other_health_measures: Default::default(),
        }
    }
}
impl RandomSets {
    /// Loads all the locations for the programs
    #[tracing::instrument]
    pub async fn load_locations(&mut self, db: &PgPool) -> anyhow::Result<()> {
        self.r_locations = Locations::find_all_in_program(Programs::RHWP, db).await?;
        self.m_locations = Locations::find_all_in_program(Programs::MHWP, db).await?;

        Ok(())
    }
    pub fn random_behavioral_risks_identified(&mut self) -> Option<String> {
        Some(
            self.behbehavioral_risks_identified
                .choose(&mut self.rand)
                .unwrap()
                .clone(),
        )
    }
    pub fn random_glucose_test(&mut self, participant: i32) -> (bool, Option<f32>, Option<bool>) {
        if !self.rand_bool(0.80) {
            return (false, None, None);
        }
        let fasted = self.rand_bool(0.5);

        let glucose_level = if self.extended_patient_info[&participant].has_diabetes {
            if fasted {
                if self.rand_bool(0.5) {
                    // They fasted for 8+ hours
                    Some(self.rand.random_range(120..300) as f32)
                } else {
                    Some(self.rand.random_range(200..300) as f32)
                }
            } else {
                // Just ate
                Some(self.rand.random_range(220..300) as f32)
            }
        } else if fasted {
            if self.rand_bool(0.5) {
                // They fasted for 8+ hours
                Some(self.rand.random_range(80..100) as f32)
            } else {
                Some(self.rand.random_range(120..140) as f32)
            }
        } else {
            // Just ate
            Some(self.rand.random_range(170..200) as f32)
        };
        (true, glucose_level, Some(fasted))
    }
    pub fn first_week_and_numer_of_case_notes(&mut self) -> (NaiveDate, i64) {
        let number_of_case_notes = self.rand.random_range(2..15);

        let start_date = match self.rand.random_range(0..100) {
            0..50 => {
                // 50% change of being months ago

                Local::now().date_naive()
                    - chrono::Duration::weeks(self.rand.random_range(1..12) * 4)
            }
            50..75 => {
                // 25% of being weeks ago
                Local::now().date_naive() - chrono::Duration::weeks(number_of_case_notes)
            }
            _ => self.rand.random_date_in_year(2024),
        };

        let earliest_possible_date =
            Local::now().date_naive() - chrono::Duration::weeks(number_of_case_notes);

        if start_date < earliest_possible_date {
            warn!(
                ?start_date,
                ?earliest_possible_date,
                "Start date is before the earliest possible date. Trying again"
            );
            self.first_week_and_numer_of_case_notes()
        } else {
            (start_date, number_of_case_notes)
        }
    }
    pub fn random_health_overview(&mut self) -> NewHealthOverview {
        let height = match self.rand.random_range(0..100) {
            0..5 => None,
            5..80 => Some(self.rand.random_range(50..75)),
            _ => Some(self.rand.random_range(75..84)),
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
    pub fn random_demographics(&mut self, participant_id: i32) -> NewDemographics {
        let extended_info = &self.extended_patient_info[&participant_id];
        let is_veteran = self.rand.random_bool(0.1);
        let ethnicity_none_or_not = match self.rand.random_range(0..100) {
            0..50 => None,
            _ => Some(Ethnicity::NotHispanicOrLatino),
        };
        let (race, race_other, race_multiple, ethnicity) = match self.rand.random_range(0..100) {
            0..40 => (Some(vec![Race::White]), None, None, ethnicity_none_or_not),
            40..50 => (Some(vec![Race::Asian]), None, None, ethnicity_none_or_not),
            50..65 => (Some(vec![Race::Black]), None, None, ethnicity_none_or_not),
            65..70 => (
                Some(vec![Race::Hispanic]),
                None,
                None,
                Some(Ethnicity::HispanicOrLatino),
            ),
            70..90 => (
                Some(vec![Race::IdentifyOther]),
                Some("Other".to_string()),
                None,
                ethnicity_none_or_not,
            ),
            90..95 => (
                Some(vec![Race::Multiracial, Race::White, Race::Black]),
                None,
                Some("White, Black".to_string()),
                ethnicity_none_or_not,
            ),
            _ => (
                Some(vec![Race::White, Race::Black]),
                None,
                None,
                ethnicity_none_or_not,
            ),
        };
        let health_insurance = match self.rand.random_range(0..100) {
            0..50 => vec![HealthInsurance::Medicaid],
            50..75 => vec![HealthInsurance::Medicare],
            75..90 => vec![HealthInsurance::Private],
            _ => vec![],
        };

        let highest_education_level = match self.rand.random_range(0..100) {
            0..50 => None,
            50..75 => Some(cs25_303_core::red_cap::EducationLevel::HighschoolOrGED),
            75..90 => Some(cs25_303_core::red_cap::EducationLevel::Associates),
            _ => Some(cs25_303_core::red_cap::EducationLevel::Bachelors),
        };
        // 10% chance of the age not being here to simulate missing data
        let age = match self.rand.random_range(0..100) {
            0..5 => None,
            _ => Some(extended_info.age),
        };
        let language = match self.rand.random_range(0..100) {
            0..10 => None,
            10..15 => Some(PreferredLanguage::Asl),
            15..20 => Some(PreferredLanguage::Other("English".to_string())),
            _ => {
                if ethnicity == Some(Ethnicity::HispanicOrLatino) {
                    Some(PreferredLanguage::Spanish)
                } else {
                    Some(PreferredLanguage::EnUs)
                }
            }
        };

        NewDemographics {
            age,
            gender: Some(extended_info.gender.clone()),
            is_veteran: Some(is_veteran),
            race,
            race_other,
            race_multiracial_other: race_multiple,
            health_insurance,
            highest_education_level,
            language,
            ethnicity,
        }
    }
    pub fn random_medications(&mut self) -> Vec<NewMedication> {
        let number_of_meds = self.rand.random_range(0..10);

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
            meds.push(random.create_new_medication(&mut self.rand));
        }
        meds
    }
    pub fn random_goals(&mut self) -> Vec<(NewParticipantGoal, Vec<NewParticipantGoalsSteps>)> {
        let number_of_goals = self.rand.random_range(0..3);
        info!(?number_of_goals, "Creating goals");
        let mut goals = Vec::with_capacity(number_of_goals);
        for _ in 0..number_of_goals {
            let random = self.goals.choose(&mut self.rand).unwrap();
            goals.push(random.create_new_goal(&mut self.rand));
        }
        goals
    }
    pub fn pick_random_program(&mut self) -> Programs {
        if self.rand.random_bool(1f64 / 3f64) {
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
        if self.rand.random_bool(0.5) {
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
        match rand::rng().random_range(0..100) {
            0..10 => Some(VisitType::OnsiteAndHome),
            _ => Some(VisitType::Onsite),
        }
    }
    pub fn random_blood_pressure(&mut self, participant: i32) -> Vec<NewBloodPressure> {
        let mut bps = Vec::with_capacity(3);
        let should_add_stand = self.rand_bool(0.25);
        if self.extended_patient_info[&participant].has_high_blood_pressure {
            bps.push(NewBloodPressure {
                blood_pressure_type: BloodPressureType::Sit,
                systolic: self.rand.random_range(130..180) as i16,
                diastolic: self.rand.random_range(80..120) as i16,
            });
            if should_add_stand {
                bps.push(NewBloodPressure {
                    blood_pressure_type: BloodPressureType::Stand,
                    systolic: self.rand.random_range(130..180) as i16,
                    diastolic: self.rand.random_range(80..120) as i16,
                });
            }
        } else {
            bps.push(NewBloodPressure {
                blood_pressure_type: BloodPressureType::Sit,
                systolic: self.rand.random_range(90..120) as i16,
                diastolic: self.rand.random_range(60..80) as i16,
            });
            if should_add_stand {
                bps.push(NewBloodPressure {
                    blood_pressure_type: BloodPressureType::Stand,
                    systolic: self.rand.random_range(90..120) as i16,
                    diastolic: self.rand.random_range(60..80) as i16,
                });
            }
        }
        if self.extended_patient_info[&participant].has_blood_pressure_cuff {
            bps.push(NewBloodPressure {
                blood_pressure_type: BloodPressureType::Personal,
                systolic: self.rand.random_range(130..180) as i16,
                diastolic: self.rand.random_range(80..120) as i16,
            });
        }
        bps
    }
    pub fn weight_for_participant(&mut self, participant: i32) -> Option<f32> {
        if self.rand_bool(0.1) {
            return None;
        }
        Some(self.extended_patient_info[&participant].weight_entry(&mut self.rand))
    }
    fn rand_bool(&mut self, chance: f64) -> bool {
        self.rand.random_bool(chance)
    }
    pub fn create_extended_profile_for_partiicpant(
        &mut self,
        participant: i32,
        gender: Gender,
    ) -> ParticipantExtendedInfo {
        // About 47% chance of having high blood pressure
        let has_high_blood_pressure = self.rand_bool(0.47);
        let has_diabetes = self.rand_bool(0.1);
        let weight_class = match self.rand.random_range(0..10) {
            0..2 => WeightCategory::Underweight,
            2..8 => WeightCategory::Overweight,
            _ => WeightCategory::Normal,
        };

        let extended = ParticipantExtendedInfo {
            has_high_blood_pressure,
            has_blood_pressure_cuff: self.rand_bool(0.5),
            has_diabetes,
            gender,
            age: self.rand.random_range(18..85) as i16,
            weight_category: weight_class,
        };
        self.extended_patient_info
            .insert(participant, extended.clone());
        extended
    }

    pub fn random_status(&mut self) -> Status {
        match self.rand.random_range(0..100) {
            0..75 => Status::Active,
            75..85 => Status::Inactive,
            85..95 => Status::NoValidContactStatus,
            _ => Status::Deceases,
        }
    }
}
