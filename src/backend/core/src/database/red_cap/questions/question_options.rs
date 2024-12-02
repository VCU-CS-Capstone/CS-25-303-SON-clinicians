use derive_more::derive::From;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::QuestionType;
/// Additional Settings for Options in a question
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AdditionalOptionSettings {
    /// If it triggers the other option. Which is a text field
    ///
    /// Should only be used if question has a string_id_other
    ///
    /// Note: If no option triggers the other option. But you have an other option id.
    ///
    /// A default Option Value will be displayed to the user
    pub triggers_other: Option<bool>,
    /// If the option is unique
    ///
    /// This is used for multi check boxes
    ///
    /// Making sure if this option is selected no other option can be selected
    ///
    /// Ignored if question is a radio
    ///
    /// Because Radios are always unique
    pub unique: Option<bool>,
}
/// Additional Settings for a question
#[derive(Debug, Clone, PartialEq, From, Serialize, Deserialize, ToSchema)]
#[schema(examples(
    additional_options_examples::boolean,
    additional_options_examples::text,
    additional_options_examples::number,
    additional_options_examples::float
))]
#[serde(tag = "type", content = "value")]
pub enum AdditionalQuestionSettings {
    /// Boolean Options
    Boolean(BooleanQuestionSettings),
    /// Text Options
    Text(TextQuestionSettings),
    /// Number Options
    Number(NumberSettings),
    /// Float Options
    Float(FloatSettings),
}
mod additional_options_examples {
    use super::AdditionalQuestionSettings;

    pub fn boolean() -> AdditionalQuestionSettings {
        AdditionalQuestionSettings::Boolean(Default::default())
    }
    pub fn text() -> AdditionalQuestionSettings {
        AdditionalQuestionSettings::Text(Default::default())
    }
    pub fn number() -> AdditionalQuestionSettings {
        AdditionalQuestionSettings::Number(Default::default())
    }
    pub fn float() -> AdditionalQuestionSettings {
        AdditionalQuestionSettings::Float(Default::default())
    }
}
impl AdditionalQuestionSettings {
    /// Validates the QuestionType matches the AdditionalQuestionOptions type
    pub fn is_of_type(&self, question_type: QuestionType) -> bool {
        match self {
            AdditionalQuestionSettings::Boolean(_) => question_type == QuestionType::Boolean,
            AdditionalQuestionSettings::Text(_) => question_type == QuestionType::Text,
            AdditionalQuestionSettings::Number(_) => question_type == QuestionType::Number,
            AdditionalQuestionSettings::Float(_) => question_type == QuestionType::Float,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct BooleanQuestionSettings {
    /// Sometimes Red Cap uses 2 for true
    /// This exists so we can handle that
    pub true_value: usize,
    pub false_value: usize,
    /// The displayed name for true
    pub true_name: Option<String>,
    /// The description for true
    pub true_description: Option<String>,
    /// The displayed name for false
    pub false_name: Option<String>,
    /// The description for false
    pub false_description: Option<String>,
}
impl Default for BooleanQuestionSettings {
    fn default() -> Self {
        Self {
            true_value: 1,
            false_value: 0,
            true_name: None,
            true_description: None,
            false_name: None,
            false_description: None,
        }
    }
}
/// Text Box Size
///
/// This is used to determine the size of the text box
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum TextBoxSize {
    /// Single line text box
    SingleLine,
    /// Multi line text box
    MultiLine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(default)]
pub struct TextQuestionSettings {
    /// The size of the text box
    ///
    /// Default should be a growable text box but starting with a single line
    pub text_box_size: Option<TextBoxSize>,
    /// If the text box should allow pretty formatting
    ///
    /// Such as Markdown or HTML
    ///
    /// Default is true
    pub allow_pretty_formatting: bool,
    /// The character limit for the text box
    ///
    /// Default is no limit
    pub character_limit: Option<usize>,
}

impl Default for TextQuestionSettings {
    fn default() -> Self {
        Self {
            text_box_size: Default::default(),
            allow_pretty_formatting: true,
            character_limit: None,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct NumberSettings {
    /// The minimum value for the number
    pub min: Option<i32>,
    /// If the minimum value is inclusive
    pub min_inclusive: bool,
    /// The maximum value for the number
    pub max: Option<i32>,
    /// If the maximum value is inclusive
    pub max_inclusive: bool,
}
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct FloatSettings {
    /// The minimum value for the number
    pub min: Option<f64>,
    /// If the minimum value is inclusive
    pub min_inclusive: bool,
    /// The maximum value for the number
    pub max: Option<f64>,
    /// If the maximum value is inclusive
    pub max_inclusive: bool,
}
