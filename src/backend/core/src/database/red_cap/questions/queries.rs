use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use utoipa::ToSchema;

use super::{AdditionalQuestionSettings, QuestionOptions, QuestionType};
/// The overview of a question
///
/// This is used to display the question in the UI
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(examples(overview_examples::text, overview_examples::radio))]
pub struct QuestionOverview {
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
    /// The description of the question
    pub description: Option<String>,
    /// If the question is required
    /// Will be ignored if requirements are not met
    pub required: bool,
    /// If the question is removed
    pub removed: bool,
    /// The requirements for the question
    ///
    /// This is a DSL that is used to determine if the question should be shown
    pub requirements: Option<String>,
    /// The options for the question
    ///
    /// This is only used for questions that have options
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<QuestionOptions>,
    /// Additional options for the question
    #[schema(value_type = Option<AdditionalQuestionSettings>)]
    pub additional_options: Option<Json<AdditionalQuestionSettings>>,
}

mod overview_examples {
    use sqlx::types::Json;

    use crate::database::red_cap::questions::{
        QuestionOptions, QuestionType, TextBoxSize, TextQuestionSettings,
    };

    use super::QuestionOverview;

    pub fn text() -> QuestionOverview {
        QuestionOverview {
            id: 1,
            category_id: 1,
            string_id: "name".to_string(),
            question_type: QuestionType::Text,
            question: "What is your name?".to_string(),
            description: Some("This is a test question".to_string()),
            required: true,
            additional_options: Some(Json(
                TextQuestionSettings {
                    text_box_size: Some(TextBoxSize::SingleLine),
                    allow_pretty_formatting: true,
                    character_limit: Some(255),
                }
                .into(),
            )),
            ..Default::default()
        }
    }

    pub fn radio() -> QuestionOverview {
        QuestionOverview {
            id: 2,
            category_id: 1,
            string_id: "feeling_today".to_string(),
            string_id_other: Some("feeling_today_other".to_string()),
            question_type: QuestionType::MultiCheckBox,
            question: "How are you feeling today?".to_string(),
            required: true,
            options: vec![
                QuestionOptions {
                    question_id: 2,
                    string_id: Some("happy".to_string()),
                    name: "Happy".to_string(),
                    ..Default::default()
                },
                QuestionOptions {
                    question_id: 2,
                    string_id: Some("sad".to_string()),
                    name: "Sad".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }
}
