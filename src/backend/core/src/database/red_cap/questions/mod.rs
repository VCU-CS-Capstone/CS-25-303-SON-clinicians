//! Due to the amount of data that can be put into red cap.
//! Questions that do not need to be answered at all times or have conditional requirements are stored using a question system.
//!
//! This prevents a ton of tables and columns with null values.
//!
//!
pub mod requirements;
use crate::database::prelude::*;
use cs25_303_macros::Columns;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use strum::{Display, EnumIs};
use thiserror::Error;
use tracing::error;
pub mod default;
pub mod new;
#[derive(Debug, Error)]
pub enum QuestionError {
    #[error("Question Not Found By String Id: {0}")]
    QuestionNotFoundByStringId(String),

    #[error("Additional Options do not match the question type {0}")]
    AdditionalOptionsMismatch(QuestionType),

    #[error("Unexpected Type")]
    UnexpectedType(),

    #[error("Option Not Found By String Id: {0}")]
    OptionNotFoundByStringId(String),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdditionalQuestionOptions {
    Boolean(BooleanQuestionOptions),
}
impl AdditionalQuestionOptions {
    pub fn is_of_type(&self, question_type: QuestionType) -> bool {
        match self {
            AdditionalQuestionOptions::Boolean(_) => question_type == QuestionType::Boolean,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BooleanQuestionOptions {
    /// Sometimes Red Cap uses 2 for true
    pub true_value: usize,
    pub false_value: usize,
    pub true_name: Option<String>,
    pub false_name: Option<String>,
}
impl Default for BooleanQuestionOptions {
    fn default() -> Self {
        Self {
            true_value: 1,
            false_value: 0,
            true_name: None,
            false_name: None,
        }
    }
}
/// Where does the question belong to
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "VARCHAR")]
pub enum QuestionForm {
    /// Case Notes
    CaseNotes,
    /// Participant Info
    ParticipantInfo,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIs, Serialize, Deserialize, Type, Display)]
#[sqlx(type_name = "VARCHAR")]
pub enum QuestionType {
    MultiCheckBox,
    Radio,
    Text,
    Number,
    Float,
    Boolean,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, Columns)]

pub struct QuestionCategory {
    pub id: i32,
    pub form: QuestionForm,
    /// The string id of the category
    pub string_id: String,
    /// The name of the category
    pub name: String,
    /// Category description
    pub description: Option<String>,
}
impl QuestionCategory {
    pub async fn delete_all(conn: &PgPool) -> DBResult<()> {
        let _ = sqlx::query("DELETE FROM question_categories")
            .execute(conn)
            .await?;
        Ok(())
    }
}
impl TableType for QuestionCategory {
    type Columns = QuestionCategoryColumn;
    fn table_name() -> &'static str
    where
        Self: Sized,
    {
        "question_categories"
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, Columns)]

pub struct Question {
    pub id: i32,
    /// The category the question belongs to
    pub category_id: i32,
    /// MUST CORRESPOND TO RED CAP ID
    pub string_id: String,
    /// For MultiCheckBox and Radio only
    ///
    /// MUST CORRESPOND TO RED CAP ID
    pub string_id_other: Option<String>,
    /// The type of question
    pub question_type: QuestionType,
    /// The name of the question
    pub question: String,
    pub description: Option<String>,
    /// If the question is required
    /// Will be ignored if requirements are not met
    pub required: bool,
    /// If the question is removed
    pub removed: bool,
    pub requirements: Option<String>,
    pub additional_options: Option<Json<AdditionalQuestionOptions>>,
}
impl Question {
    pub async fn find_by_string_id(red_cap_id: &str, conn: &PgPool) -> DBResult<Option<Self>> {
        let question = SimpleSelectQueryBuilder::new(Self::table_name(), &QuestionColumn::all())
            .where_equals(QuestionColumn::StringId, red_cap_id)
            .limit(1)
            .query_as::<Self>()
            .fetch_optional(conn)
            .await?;
        Ok(question)
    }
    pub async fn find_by_string_id_or_other(
        red_cap_id: &str,
        other_id: &str,
        conn: &PgPool,
    ) -> DBResult<Option<Self>> {
        let question = sqlx::query_as(
            "
            SELECT * FROM questions
            WHERE string_id = $1 OR string_id_other = $1
            LIMIT 1
            ",
        )
        .bind(red_cap_id)
        .bind(other_id)
        .fetch_optional(conn)
        .await?;
        Ok(question)
    }
    pub async fn get_all_in_category(category_id: i32, conn: &PgPool) -> DBResult<Vec<Self>> {
        let questions = SimpleSelectQueryBuilder::new(Self::table_name(), &QuestionColumn::all())
            .where_equals(QuestionColumn::CategoryId, category_id)
            .query_as::<Self>()
            .fetch_all(conn)
            .await?;
        Ok(questions)
    }
}
impl TableType for Question {
    type Columns = QuestionColumn;
    fn table_name() -> &'static str
    where
        Self: Sized,
    {
        "questions"
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionOptions {
    pub triggers_other: Option<bool>,
    pub unique: Option<bool>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, Columns)]
pub struct QuestionOptions {
    pub id: i32,
    pub question_id: i32,
    /// A id that is used to reference the option
    pub string_id: Option<String>,
    /// The index of the option in red cap
    pub red_cap_option_index: Option<i32>,
    /// The name of the option
    pub name: String,
    /// Additional description
    pub description: Option<String>,
    /// Was the response removed
    pub removed: bool,
    pub additional_options: Option<Json<OptionOptions>>,
}
impl QuestionOptions {
    pub async fn find_option_with_string_id_and_in_question(
        string_id: &str,
        question_id: i32,
        conn: &PgPool,
    ) -> DBResult<Option<Self>> {
        let option =
            SimpleSelectQueryBuilder::new(Self::table_name(), &QuestionOptionsColumn::all())
                .where_equals_then(QuestionOptionsColumn::StringId, string_id, |builder| {
                    builder.and_equals(QuestionOptionsColumn::QuestionId, question_id);
                })
                .limit(1)
                .query_as::<Self>()
                .fetch_optional(conn)
                .await?;
        Ok(option)
    }
    pub async fn find_option_with_red_cap_index_and_in_question(
        red_cap_index: i32,
        question_id: i32,
        conn: &PgPool,
    ) -> DBResult<Option<Self>> {
        let option =
            SimpleSelectQueryBuilder::new(Self::table_name(), &QuestionOptionsColumn::all())
                .where_equals_then(
                    QuestionOptionsColumn::RedCapOptionIndex,
                    red_cap_index,
                    |builder| {
                        builder.and_equals(QuestionOptionsColumn::QuestionId, question_id);
                    },
                )
                .limit(1)
                .query_as::<Self>()
                .fetch_optional(conn)
                .await?;
        Ok(option)
    }
}
impl TableType for QuestionOptions {
    type Columns = QuestionOptionsColumn;
    fn table_name() -> &'static str
    where
        Self: Sized,
    {
        "question_options"
    }
}

/// The value of the question
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum QuestionDataValue<O = QuestionOptions, R = QuestionOptions> {
    /// String Value
    Text(String),
    /// Number Value
    Number(i32),
    /// Float Value
    Float(f32),
    /// Boolean Value
    Boolean(bool),
    /// Multi Check Box
    MultiCheckBox {
        /// Options that are selected
        options: Vec<O>,
        /// Other value
        #[serde(skip_serializing_if = "Option::is_none")]
        other: Option<String>,
    },
    Radio {
        /// The selected option
        option: R,
        /// Other value
        #[serde(skip_serializing_if = "Option::is_none")]
        other: Option<String>,
    },
}
impl QuestionDataValue<QuestionOptions, QuestionOptions> {
    /// Turns the value into a multi check box (As Long as it is not a multi check box)
    ///
    /// Then adds the other value and options
    ///
    /// # Returns
    /// If the value was successfully converted
    /// If the value was already a multi check box it will return false
    pub fn make_multi_check_with_other(&mut self, options: Vec<QuestionOptions>) -> bool {
        let other = match self {
            QuestionDataValue::Text(text) => std::mem::take(text),
            QuestionDataValue::Number(number) => number.to_string(),
            QuestionDataValue::Float(float) => float.to_string(),
            QuestionDataValue::Boolean(boolean) => boolean.to_string(),
            _ => {
                error!(?self, "Illegal State for Multi Check Box with Other");
                return false;
            }
        };

        *self = QuestionDataValue::MultiCheckBox {
            options,
            other: Some(other),
        };
        true
    }
    /// Adds the other value if its a multi check box or radio
    pub fn push_other_to_other(&mut self, other: String) -> bool {
        match self {
            QuestionDataValue::MultiCheckBox { other: value, .. } => {
                *value = Some(other);
                true
            }
            QuestionDataValue::Radio { other: value, .. } => {
                *value = Some(other);
                true
            }
            _ => {
                error!(?self, "Illegal State for Other");
                false
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "RECORD")]
pub struct QuestionAnswerMCB {
    pub option_id: i32,
    pub option_name: String,
    pub option_string_id: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestionAnswerRadio {
    pub option_id: i32,
    pub option_name: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "RECORD")]
pub struct DBQuestionAnswerRadio {
    pub option_id: Option<i32>,
    pub option_name: Option<String>,
}
impl From<DBQuestionAnswerRadio> for Option<QuestionAnswerRadio> {
    fn from(val: DBQuestionAnswerRadio) -> Self {
        Some(QuestionAnswerRadio {
            option_id: val.option_id?,
            option_name: val.option_name,
        })
    }
}
/// A database response for a question with answer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct DBQuestionResponse {
    pub answer_id: i64,
    pub question_id: i32,
    pub question_string_id: String,
    pub question_string_id_other: Option<String>,
    pub response_type: QuestionType,
    pub value_text: Option<String>,
    pub value_number: Option<i32>,
    pub value_float: Option<f32>,
    pub value_boolean: Option<bool>,
    pub value_radio: DBQuestionAnswerRadio,
    pub options: Vec<QuestionAnswerMCB>,
}

/// A clean response for a question with answer
///
/// This changes the value to a more usable format
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CleanQuestionResponse {
    pub answer_id: i64,
    pub question_id: i32,
    pub question_string_id: String,
    pub question_string_id_other: Option<String>,
    pub response_type: QuestionType,
    pub value: Option<QuestionDataValue<QuestionAnswerMCB, Option<QuestionAnswerRadio>>>,
}

impl From<DBQuestionResponse> for CleanQuestionResponse {
    fn from(value: DBQuestionResponse) -> Self {
        let data_value = match value.response_type {
            QuestionType::MultiCheckBox => {
                let options = value.options;
                let other = value.value_text;
                if other.is_none() && options.is_empty() {
                    None
                } else {
                    Some(QuestionDataValue::MultiCheckBox { options, other })
                }
            }
            QuestionType::Radio => {
                let option: Option<QuestionAnswerRadio> = value.value_radio.into();
                let other = value.value_text;
                if option.is_none() && other.is_none() {
                    None
                } else {
                    Some(QuestionDataValue::Radio { option, other })
                }
            }
            QuestionType::Text => value.value_text.map(QuestionDataValue::Text),
            QuestionType::Number => value.value_number.map(QuestionDataValue::Number),
            QuestionType::Float => value.value_float.map(QuestionDataValue::Float),
            QuestionType::Boolean => value.value_boolean.map(QuestionDataValue::Boolean),
        };
        Self {
            answer_id: value.answer_id,
            question_id: value.question_id,
            question_string_id: value.question_string_id,
            question_string_id_other: value.question_string_id_other,
            response_type: value.response_type,
            value: data_value,
        }
    }
}
