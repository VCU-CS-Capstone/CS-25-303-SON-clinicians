//!
//!
//! So Umm instead of using the repeating entry stuff in RedCap
//! They have a form that is up to 10 goals. Where each one could be null
//!
//! So yeah. We are just going to use a 1:many relationship
use crate::database::prelude::*;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, Executor};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NewParticipantGoal {
    pub goal: String,
    pub is_active: Option<bool>,
    pub red_cap_index: Option<i32>,
}
impl NewParticipantGoal {
    pub async fn insert_return_goal(
        self,
        participant_id: i32,
        database: impl Executor<'_, Database = sqlx::Postgres>,
    ) -> DBResult<ParticipantGoals> {
        let Self {
            goal,
            is_active,
            red_cap_index,
        } = self;
        SimpleInsertQueryBuilder::new(ParticipantGoals::table_name())
            .insert(ParticipantGoalsColumn::ParticipantId, participant_id)
            .insert(ParticipantGoalsColumn::Goal, goal)
            .insert(ParticipantGoalsColumn::IsActive, is_active)
            .insert(ParticipantGoalsColumn::RedCapIndex, red_cap_index)
            .return_all()
            .query_as()
            .fetch_one(database)
            .await
            .map_err(DBError::from)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow, Columns)]
pub struct ParticipantGoals {
    pub id: i32,
    pub participant_id: i32,
    pub goal: String,
    pub is_active: Option<bool>,
    pub red_cap_index: Option<i32>,
    pub hidden_from_red_cap: bool,
    pub created_at: DateTime<FixedOffset>,
}
impl ParticipantGoals {
    pub async fn get_goal_by_id(
        goal_id: i32,
        database: impl Executor<'_, Database = sqlx::Postgres>,
    ) -> DBResult<Option<ParticipantGoals>> {
        SimpleSelectQueryBuilder::new(
            ParticipantGoals::table_name(),
            &ParticipantGoalsColumn::all(),
        )
        .where_equals(ParticipantGoalsColumn::Id, goal_id)
        .query_as()
        .fetch_optional(database)
        .await
        .map_err(DBError::from)
    }
    pub async fn set_red_cap_index(
        &mut self,
        red_cap_index: i32,
        database: impl Executor<'_, Database = sqlx::Postgres>,
    ) -> DBResult<()> {
        if self.red_cap_index == Some(red_cap_index) {
            return Ok(());
        }
        self.red_cap_index = Some(red_cap_index);
        sqlx::query("UPDATE participant_goals SET red_cap_index = $1 WHERE id = $2")
            .bind(red_cap_index)
            .bind(self.id)
            .execute(database)
            .await?;
        Ok(())
    }
    pub async fn get_all_participant_goals(
        participant_id: i32,
        database: impl Executor<'_, Database = sqlx::Postgres>,
    ) -> DBResult<Vec<ParticipantGoals>> {
        SimpleSelectQueryBuilder::new(
            ParticipantGoals::table_name(),
            &ParticipantGoalsColumn::all(),
        )
        .where_equals(ParticipantGoalsColumn::ParticipantId, participant_id)
        .query_as()
        .fetch_all(database)
        .await
        .map_err(DBError::from)
    }

    pub async fn process_red_cap_indexes(participant_id: i32, database: &PgPool) -> DBResult<()> {
        let mut goals = Self::get_all_participant_goals(participant_id, database).await?;
        goals.sort_by(|a, b| {
            a.red_cap_index
                .unwrap_or_default()
                .cmp(&b.red_cap_index.unwrap_or_default())
        });
        for (index, goal) in goals.iter_mut().enumerate() {
            let red_cap_index = index as i32 + 1;

            if goal.red_cap_index == Some(red_cap_index) {
                continue;
            }

            goal.set_red_cap_index(red_cap_index, database).await?;
        }
        Ok(())
    }
}
impl TableType for ParticipantGoals {
    type Columns = ParticipantGoalsColumn;
    fn table_name() -> &'static str {
        "participant_goals"
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NewParticipantGoalsSteps {
    pub goal_id: Option<i32>,
    pub step: String,
    pub confidence_level: Option<i16>,
    pub date_set: Option<NaiveDate>,
    pub date_to_be_completed: Option<NaiveDate>,
    /// True == Yes
    /// False == No
    /// Select No until goal is achieved
    #[serde(default)]
    pub action_step: Option<bool>,
    pub red_cap_index: Option<i32>,
}
impl NewParticipantGoalsSteps {
    pub async fn insert_return_none(
        self,
        participant_id: i32,
        database: impl Executor<'_, Database = sqlx::Postgres>,
    ) -> DBResult<()> {
        let Self {
            goal_id,
            step,
            confidence_level,
            date_set,
            date_to_be_completed,
            action_step,
            red_cap_index,
        } = self;
        SimpleInsertQueryBuilder::new(ParticipantGoalsSteps::table_name())
            .insert(ParticipantGoalsStepsColumn::ParticipantId, participant_id)
            .insert(ParticipantGoalsStepsColumn::GoalId, goal_id)
            .insert(ParticipantGoalsStepsColumn::Step, step)
            .insert(
                ParticipantGoalsStepsColumn::ConfidenceLevel,
                confidence_level,
            )
            .insert(ParticipantGoalsStepsColumn::DateSet, date_set)
            .insert(
                ParticipantGoalsStepsColumn::DateToBeCompleted,
                date_to_be_completed,
            )
            .insert(ParticipantGoalsStepsColumn::ActionStep, action_step)
            .insert(ParticipantGoalsStepsColumn::RedCapIndex, red_cap_index)
            .query()
            .execute(database)
            .await?;
        Ok(())
    }
    /// Uses the the argument `related_goal_id` as the goal_id
    #[inline]
    pub async fn insert_with_goal_return_none(
        mut self,
        participant_id: i32,
        related_goal_id: i32,
        database: impl Executor<'_, Database = sqlx::Postgres>,
    ) -> DBResult<()> {
        self.goal_id = Some(related_goal_id);
        self.insert_return_none(participant_id, database).await?;
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow, Columns)]
pub struct ParticipantGoalsSteps {
    pub id: i32,
    pub goal_id: Option<i32>,
    pub participant_id: i32,
    /// This is technically nullable. But if it is null. I will not put it in the database
    pub step: String,
    pub confidence_level: Option<i16>,
    pub date_set: Option<NaiveDate>,
    pub date_to_be_completed: Option<NaiveDate>,
    /// True == Yes
    /// False == No
    /// Select No until goal is achieved
    pub action_step: Option<bool>,
    pub red_cap_index: Option<i32>,
    pub hidden_from_red_cap: bool,
    pub created_at: chrono::DateTime<FixedOffset>,
}
impl ParticipantGoalsSteps {
    pub async fn set_red_cap_index(
        &mut self,
        red_cap_index: i32,
        database: impl Executor<'_, Database = sqlx::Postgres>,
    ) -> DBResult<()> {
        if self.red_cap_index == Some(red_cap_index) {
            return Ok(());
        }
        self.red_cap_index = Some(red_cap_index);
        sqlx::query("UPDATE participant_goal_steps SET red_cap_index = $1 WHERE id = $2")
            .bind(red_cap_index)
            .bind(self.id)
            .execute(database)
            .await?;
        Ok(())
    }

    pub async fn get_all_participant_goals_steps(
        participant_id: i32,
        database: impl Executor<'_, Database = sqlx::Postgres>,
    ) -> DBResult<Vec<ParticipantGoalsSteps>> {
        SimpleSelectQueryBuilder::new(
            ParticipantGoalsSteps::table_name(),
            &ParticipantGoalsStepsColumn::all(),
        )
        .where_equals(ParticipantGoalsStepsColumn::ParticipantId, participant_id)
        .query_as()
        .fetch_all(database)
        .await
        .map_err(DBError::from)
    }

    pub async fn process_red_cap_indexes(participant_id: i32, database: &PgPool) -> DBResult<()> {
        let mut goals = Self::get_all_participant_goals_steps(participant_id, database).await?;
        goals.sort_by(|a, b| {
            a.red_cap_index
                .unwrap_or_default()
                .cmp(&b.red_cap_index.unwrap_or_default())
        });
        for (index, goal) in goals.iter_mut().enumerate() {
            let red_cap_index = index as i32 + 1;

            if goal.red_cap_index == Some(red_cap_index) {
                continue;
            }

            goal.set_red_cap_index(red_cap_index, database).await?;
        }
        Ok(())
    }
}
impl TableType for ParticipantGoalsSteps {
    type Columns = ParticipantGoalsStepsColumn;
    fn table_name() -> &'static str {
        "participant_goal_steps"
    }
}
