use super::utils::RandDate;
use chrono::{Duration, Local, NaiveDate, Weekday};
use cs25_303_core::{
    database::red_cap::participants::{
        goals::{NewParticipantGoal, NewParticipantGoalsSteps},
        NewMedication,
    },
    red_cap::{Gender, MedicationFrequency},
};
use rand::{seq::IndexedRandom, Rng};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, info};
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RandomDateOptions {
    pub min: Option<NaiveDate>,
    pub max: Option<NaiveDate>,
}
impl RandomDateOptions {
    pub fn random_date(&self, rand: &mut impl Rng) -> NaiveDate {
        let Self { min, max } = self;
        let min = min.unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        let max = max.unwrap_or_else(|| Local::now().date_naive());
        rand.random_date_with_range(&min, &max)
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "random", content = "options")]
pub enum RandomValue {
    Array(Vec<Value>),
    Number { min: i32, max: i32 },
    Bool,
    Date(Option<RandomDateOptions>),
}
impl RandomValue {
    pub fn random_string_from_options(&self, rand: &mut impl Rng) -> String {
        match self {
            RandomValue::Array(options) => {
                let value = options.choose(rand).unwrap();
                value.as_str().unwrap_or_default().to_owned()
            }
            RandomValue::Number { min, max } => rand.random_range(*min..*max).to_string(),
            RandomValue::Bool => rand.random_bool(0.5).to_string(),
            RandomValue::Date(value) => {
                let value = value.clone().unwrap_or_default();
                value.random_date(rand).to_string()
            }
        }
    }
    pub fn date(&self, rand: &mut impl Rng) -> NaiveDate {
        match self {
            RandomValue::Date(value) => {
                let value = value.clone().unwrap_or_default();
                value.random_date(rand)
            }
            _ => {
                error!("Not a date");
                Local::now().date_naive()
            }
        }
    }
    pub fn random_i16(&self, rand: &mut impl Rng) -> i16 {
        match self {
            RandomValue::Number { min, max } => rand.random_range(*min..*max) as i16,
            _ => {
                error!("Not a number");
                rand.random_range(0..100) as i16
            }
        }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ValueOrRandom<T> {
    Value(T),
    Random(RandomValue),
}
impl ValueOrRandom<i16> {
    pub fn i16_value(&self, rand: &mut impl Rng) -> i16 {
        match self {
            ValueOrRandom::Value(value) => *value,
            ValueOrRandom::Random(random_value) => random_value.random_i16(rand),
        }
    }
}
impl ValueOrRandom<String> {
    pub fn string_value(&self, rand: &mut impl Rng) -> String {
        match self {
            ValueOrRandom::Value(value) => value.clone(),
            ValueOrRandom::Random(random_value) => random_value.random_string_from_options(rand),
        }
    }
}
impl ValueOrRandom<NaiveDate> {
    pub fn date_value(&self, rand: &mut impl Rng) -> NaiveDate {
        match self {
            ValueOrRandom::Value(value) => *value,
            ValueOrRandom::Random(random_value) => random_value.date(rand),
        }
    }
}
impl<T> ValueOrRandom<T>
where
    T: From<String> + Clone,
{
    pub fn value_from_string(&self, rand: &mut impl Rng) -> T {
        match self {
            ValueOrRandom::Value(value) => value.clone(),
            ValueOrRandom::Random(random_value) => {
                let string_value = random_value.random_string_from_options(rand);
                T::from(string_value)
            }
        }
    }
}
/// Random Base Participant
///
/// Just the first name and last name and the gender
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RandomParticipant {
    pub first_name: String,
    pub last_name: String,
    pub gender: Gender,
}
/// Random Complete Goal
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RandomCompleteGoal {
    pub goal: RandomGoal,
    pub steps: Vec<RandomGoalStep>,
}
impl RandomCompleteGoal {
    pub fn create_new_goal(
        &self,
        rand: &mut impl Rng,
    ) -> (NewParticipantGoal, Vec<NewParticipantGoalsSteps>) {
        let goal = self.goal.create_new_goal();

        let random_step: &RandomGoalStep = self.steps.choose(rand).unwrap();
        let steps = vec![random_step.create_new_goal_step(rand)];
        (goal, steps)
    }
}
/// Random Goal
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RandomGoal {
    pub goal: String,
    pub is_active: bool,
}
impl RandomGoal {
    pub fn create_new_goal(&self) -> NewParticipantGoal {
        let RandomGoal { goal, is_active } = self;
        NewParticipantGoal {
            goal: goal.clone(),
            is_active: Some(*is_active),
            ..Default::default()
        }
    }
}
/// Random Goal Step
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RandomGoalStep {
    pub step: String,
}
impl RandomGoalStep {
    pub fn create_new_goal_step(&self, rand: &mut impl Rng) -> NewParticipantGoalsSteps {
        let RandomGoalStep { step } = self;
        let confidence_in_achieving = rand.random_range(1..=10);
        let date_set = rand.random_date_in_year(2024);
        let date_to_be_achieved = rand.random_date_in_year(2024);
        NewParticipantGoalsSteps {
            goal_id: None,
            step: step.clone(),
            confidence_level: Some(confidence_in_achieving),
            date_set: Some(date_set),
            date_to_be_completed: Some(date_to_be_achieved),
            ..Default::default()
        }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RandomMedication {
    pub name: String,
    pub dosage: ValueOrRandom<String>,
    pub frequency: ValueOrRandom<MedicationFrequency>,
}
impl RandomMedication {
    pub fn create_new_medication(&self, rand: &mut impl Rng) -> NewMedication {
        let RandomMedication {
            name,
            dosage,
            frequency,
        } = self;

        let dosage = dosage.string_value(rand);
        let freqeuency = frequency.value_from_string(rand);
        let start_date = Some(rand.random_date_in_year(2024));
        let (is_current, date_discontinued) = match rand.random_range(0..100) {
            0..75 => (Some(true), None),
            75..95 => (Some(false), Some(Local::now().date_naive())),
            _ => (None, None),
        };
        let date_prescribed = if rand.random_bool(0.75) {
            start_date.as_ref().map(|date| *date - Duration::days(30))
        } else {
            None
        };
        info!(
            ?date_discontinued,
            ?is_current,
            ?date_prescribed,
            "Medication"
        );
        NewMedication {
            name: name.clone(),
            dosage: Some(dosage),
            frequency: Some(freqeuency),
            date_prescribed: date_prescribed,
            date_entered_into_system: start_date,
            is_current,
            date_discontinued,
            ..Default::default()
        }
    }
}
