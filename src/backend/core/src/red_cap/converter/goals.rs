use chrono::NaiveDate;

use crate::{
    database::red_cap::participants::goals::{
        NewParticipantGoal, ParticipantGoals, ParticipantGoalsSteps,
    },
    red_cap::RedCapDataSet,
};

use super::{RedCapConverter, RedCapConverterError};
// 1-9
#[derive(Debug, Clone, PartialEq)]
pub struct RedCapGoals {
    // Format ltg{number} 1-9
    pub goal: String,
    // Format active_ltg{number} 1-9
    pub is_active: Option<bool>,
    pub red_cap_index: i32,
}
impl From<RedCapGoals> for NewParticipantGoal {
    fn from(value: RedCapGoals) -> Self {
        let RedCapGoals {
            goal,
            is_active,
            red_cap_index,
        } = value;
        Self {
            goal,
            is_active,
            red_cap_index: Some(red_cap_index),
        }
    }
}
impl From<ParticipantGoals> for RedCapGoals {
    fn from(value: ParticipantGoals) -> Self {
        let ParticipantGoals {
            goal,
            is_active,
            red_cap_index,
            ..
        } = value;
        Self {
            goal,
            is_active,
            red_cap_index: red_cap_index.unwrap_or_default(),
        }
    }
}
impl RedCapGoals {
    pub fn read_index<D: RedCapDataSet>(data: &D, index: i32) -> Option<Self> {
        let goal = format!("ltg{}", index);
        let goal = data.get_string(&goal)?;
        let is_active = format!("active_ltg{}", index);
        let is_active = data.get_bool(&is_active);
        Some(Self {
            goal,
            is_active,
            red_cap_index: index,
        })
    }
    pub fn read<D: RedCapDataSet>(data: &D) -> Vec<Self> {
        (1..10).filter_map(|x| Self::read_index(data, x)).collect()
    }
    pub fn write<D: RedCapDataSet>(&self, data: &mut D) {
        let goal = format!("ltg{}", self.red_cap_index);
        let is_active = format!("active_ltg{}", self.red_cap_index);
        data.insert(goal, self.goal.clone().into());
        data.insert(is_active, self.is_active.into());
        if self.red_cap_index > 1 {
            data.insert(format!("newltg{}", self.red_cap_index), 1.into());
        }
    }
}
//  1 -25
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RedCapGoalsSteps {
    //  associated_goal_{number}
    pub associated_goal: Option<i32>,
    // goal{number}
    pub step: String,
    // confidence{number}
    pub confidence_level: Option<i16>,
    // set{number}
    pub date_set: Option<NaiveDate>,
    // date{number}
    pub date_to_be_completed: Option<NaiveDate>,
    // action{number}
    pub action_step: Option<bool>,
    pub red_cap_index: i32,
}

impl RedCapGoalsSteps {
    pub async fn from_db(
        step: ParticipantGoalsSteps,
        converter: &mut RedCapConverter,
    ) -> Result<Self, RedCapConverterError> {
        let ParticipantGoalsSteps {
            step,
            confidence_level,
            date_set,
            date_to_be_completed,
            action_step,
            red_cap_index,
            goal_id,
            ..
        } = step;

        let associated_goal = if let Some(goal_id) = goal_id {
            let goal = ParticipantGoals::get_goal_by_id(goal_id, &converter.database).await?;
            goal.and_then(|x| x.red_cap_index)
        } else {
            None
        };
        let result = Self {
            associated_goal,
            step,
            confidence_level,
            date_set,
            date_to_be_completed,
            action_step,
            red_cap_index: red_cap_index.unwrap_or_default(),
        };
        Ok(result)
    }
    pub fn read_index<D: RedCapDataSet>(data: &D, index: i32) -> Option<Self> {
        let goal = format!("goal{}", index);
        let confidence = format!("confidence{}", index);
        let associated_goal = format!("associated_goal_{}", index);
        let set = format!("set{}", index);
        let date = format!("date{}", index);
        let action = format!("complete{}", index);

        let step = data.get_string(&goal)?;
        let associated_goal = data.get_number(&associated_goal).map(|x| x as i32);
        let confidence_level = data.get_number(&confidence).map(|x| x as i16);
        let date_set = data.get_date(&set);
        let date_to_be_completed = data.get_date(&date);
        let action_step = data.get_bool(&action);
        Some(Self {
            associated_goal,
            step,
            confidence_level,
            date_set,
            date_to_be_completed,
            action_step,
            red_cap_index: index,
        })
    }
    pub fn read<D: RedCapDataSet>(data: &D) -> Vec<Self> {
        (1..26).filter_map(|x| Self::read_index(data, x)).collect()
    }
    pub fn write<D: RedCapDataSet>(&self, data: &mut D) {
        let associated_goal = format!("associated_goal_{}", self.red_cap_index);

        let goal = format!("goal{}", self.red_cap_index);
        let confidence = format!("confidence{}", self.red_cap_index);
        let set = format!("set{}", self.red_cap_index);
        let date = format!("date{}", self.red_cap_index);
        let action = format!("complete{}", self.red_cap_index);
        data.insert(associated_goal, self.associated_goal.into());
        data.insert(goal, self.step.clone().into());
        data.insert(confidence, self.confidence_level.into());
        data.insert(set, self.date_set.into());
        data.insert(date, self.date_to_be_completed.into());
        data.insert(action, self.action_step.into());
        if self.red_cap_index > 1 {
            data.insert(format!("newstg{}", self.red_cap_index), 1.into());
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedCapCompleteGoals {
    pub goals: Vec<RedCapGoals>,
    pub steps: Vec<RedCapGoalsSteps>,
}
impl RedCapCompleteGoals {
    pub fn read<D: RedCapDataSet>(data: &D) -> Result<Self, RedCapConverterError> {
        let goals = RedCapGoals::read(data);
        let steps = RedCapGoalsSteps::read(data);
        Ok(Self { goals, steps })
    }
}
