use chrono::{Local, NaiveDate};
use cs25_303_core::{
    database::red_cap::participants::{
        goals::{NewParticipantGoal, NewParticipantGoalsSteps},
        NewMedication,
    },
    red_cap::{Gender, MedicationFrequency},
};
use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::error;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RandomDateOptions {
    pub min: Option<NaiveDate>,
    pub max: Option<NaiveDate>,
}
impl RandomDateOptions {
    pub fn random_date(&self) -> NaiveDate {
        let Self { min, max } = self;
        let min = min.unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        let max = max.unwrap_or_else(|| Local::now().date_naive());
        let days = max.signed_duration_since(min).num_days();
        let random_days = rand::thread_rng().gen_range(0..days);
        min + chrono::Duration::days(random_days)
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
    pub fn random_string_from_options(&self) -> String {
        match self {
            RandomValue::Array(options) => {
                let value = options.choose(&mut rand::thread_rng()).unwrap();
                value.as_str().unwrap_or_default().to_owned()
            }
            RandomValue::Number { min, max } => {
                rand::thread_rng().gen_range(*min..*max).to_string()
            }
            RandomValue::Bool => rand::thread_rng().gen_bool(0.5).to_string(),
            RandomValue::Date(value) => {
                let value = value.clone().unwrap_or_default();
                value.random_date().to_string()
            }
        }
    }
    pub fn date(&self) -> NaiveDate {
        match self {
            RandomValue::Date(value) => {
                let value = value.clone().unwrap_or_default();
                value.random_date()
            }
            _ => {
                error!("Not a date");
                Local::now().date_naive()
            }
        }
    }
    pub fn random_i16(&self) -> i16 {
        match self {
            RandomValue::Number { min, max } => rand::thread_rng().gen_range(*min..*max) as i16,
            _ => {
                error!("Not a number");
                rand::thread_rng().gen_range(0..100) as i16
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
    pub fn i16_value(&self) -> i16 {
        match self {
            ValueOrRandom::Value(value) => *value,
            ValueOrRandom::Random(random_value) => random_value.random_i16(),
        }
    }
}
impl ValueOrRandom<String> {
    pub fn string_value(&self) -> String {
        match self {
            ValueOrRandom::Value(value) => value.clone(),
            ValueOrRandom::Random(random_value) => random_value.random_string_from_options(),
        }
    }
}
impl ValueOrRandom<NaiveDate> {
    pub fn date_value(&self) -> NaiveDate {
        match self {
            ValueOrRandom::Value(value) => *value,
            ValueOrRandom::Random(random_value) => random_value.date(),
        }
    }
}
impl<T> ValueOrRandom<T>
where
    T: From<String> + Clone,
{
    pub fn value_from_string(&self) -> T {
        match self {
            ValueOrRandom::Value(value) => value.clone(),
            ValueOrRandom::Random(random_value) => {
                let string_value = random_value.random_string_from_options();
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
    pub fn create_new_goal(&self) -> (NewParticipantGoal, Vec<NewParticipantGoalsSteps>) {
        let goal = self.goal.create_new_goal();

        let random_step: &RandomGoalStep = self.steps.choose(&mut rand::thread_rng()).unwrap();
        let steps = vec![random_step.create_new_goal_step()];
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
    pub confidence_in_achieving: ValueOrRandom<i16>,
    pub date_set: ValueOrRandom<NaiveDate>,
    pub date_to_be_achieved: ValueOrRandom<NaiveDate>,
}
impl RandomGoalStep {
    pub fn create_new_goal_step(&self) -> NewParticipantGoalsSteps {
        let RandomGoalStep {
            step,
            confidence_in_achieving,
            date_set,
            date_to_be_achieved,
        } = self;
        let confidence_in_achieving = confidence_in_achieving.i16_value();
        let date_set = date_set.date_value();
        let date_to_be_achieved = date_to_be_achieved.date_value();
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
    pub start_date: Option<ValueOrRandom<NaiveDate>>,
}
impl RandomMedication {
    pub fn create_new_medication(&self) -> NewMedication {
        let RandomMedication {
            name,
            dosage,
            frequency,
            start_date,
        } = self;

        let dosage = dosage.string_value();
        let freqeuency = frequency.value_from_string();
        let start_date = start_date.as_ref().map(|date| date.date_value());

        NewMedication {
            name: name.clone(),
            dosage: Some(dosage),
            frequency: Some(freqeuency),
            date_prescribed: None,
            date_entered_into_system: start_date,
            ..Default::default()
        }
    }
}
