//! Due to the amount of data that can be put into red cap. Questions that do not need to be answers at all times or have conditional requirements are stored using a question system.
use crate::database::{
    prelude::*,
    red_cap::questions::{DBQuestionResponse, QuestionDataValue, QuestionType},
};
use cs25_303_macros::Columns;
use serde::{Deserialize, Serialize};

use tracing::instrument;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow, Columns)]
pub struct CaseNoteQuestionAnswers {
    pub id: i64,
    pub case_note_id: i32,
    pub question_id: i32,
    pub response_type: QuestionType,
    pub value_text: Option<String>,
    pub value_number: Option<i32>,
    pub value_radio: Option<i32>,
    pub value_boolean: Option<bool>,
    pub value_float: Option<f32>,
}
impl CaseNoteQuestionAnswers {
    pub async fn add_multi_check_box(
        &self,
        option_id: i32,
        database: &sqlx::PgPool,
    ) -> DBResult<()> {
        SimpleInsertQueryBuilder::new(QuestionAnswerMultiCheck::table_name())
            .insert(QuestionAnswerMultiCheckColumn::QuestionAnswersId, self.id)
            .insert(QuestionAnswerMultiCheckColumn::OptionId, option_id)
            .query()
            .execute(database)
            .await?;
        Ok(())
    }
}
impl CaseNoteQuestionAnswers {
    pub async fn find_all_by_case_note_id(
        case_note_id: i32,
        database: &sqlx::PgPool,
    ) -> sqlx::Result<Vec<Self>> {
        SimpleSelectQueryBuilder::new(Self::table_name(), &CaseNoteQuestionAnswersColumn::all())
            .where_equals(CaseNoteQuestionAnswersColumn::CaseNoteId, case_note_id)
            .query_as()
            .fetch_all(database)
            .await
    }
}
impl TableType for CaseNoteQuestionAnswers {
    type Columns = CaseNoteQuestionAnswersColumn;
    fn table_name() -> &'static str {
        "case_note_question_answers"
    }
}
/// Table Name: question_answer_multi_check_box
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, Columns)]
pub struct QuestionAnswerMultiCheck {
    pub id: i64,
    pub question_answers_id: i64,
    pub option_id: i32,
}
impl TableType for QuestionAnswerMultiCheck {
    type Columns = QuestionAnswerMultiCheckColumn;
    fn table_name() -> &'static str {
        "case_note_question_answer_mcb"
    }
}
#[instrument]
pub async fn add_question(
    question_id: i32,
    case_note: i32,
    value: QuestionDataValue,
    database: &sqlx::PgPool,
) -> DBResult<()> {
    // TODO: Handle cases where a question is already answered
    let mut query = SimpleInsertQueryBuilder::new(CaseNoteQuestionAnswers::table_name());

    query
        .insert(CaseNoteQuestionAnswersColumn::CaseNoteId, case_note)
        .insert(CaseNoteQuestionAnswersColumn::QuestionId, question_id);
    match value {
        QuestionDataValue::Text(text) => {
            query
                .insert(
                    CaseNoteQuestionAnswersColumn::ResponseType,
                    QuestionType::Text,
                )
                .insert(CaseNoteQuestionAnswersColumn::ValueText, text)
                .query()
                .execute(database)
                .await?;
        }
        QuestionDataValue::Number(number) => {
            query
                .insert(
                    CaseNoteQuestionAnswersColumn::ResponseType,
                    QuestionType::Number,
                )
                .insert(CaseNoteQuestionAnswersColumn::ValueNumber, number)
                .query()
                .execute(database)
                .await?;
        }
        QuestionDataValue::Float(value) => {
            query
                .insert(
                    CaseNoteQuestionAnswersColumn::ResponseType,
                    QuestionType::Float,
                )
                .insert(CaseNoteQuestionAnswersColumn::ValueFloat, value)
                .query()
                .execute(database)
                .await?;
        }
        QuestionDataValue::Boolean(value) => {
            query
                .insert(
                    CaseNoteQuestionAnswersColumn::ResponseType,
                    QuestionType::Boolean,
                )
                .insert(CaseNoteQuestionAnswersColumn::ValueBoolean, value)
                .query()
                .execute(database)
                .await?;
        }
        QuestionDataValue::Radio { option, other } => {
            query
                .insert(
                    CaseNoteQuestionAnswersColumn::ResponseType,
                    QuestionType::Radio,
                )
                .insert(CaseNoteQuestionAnswersColumn::ValueText, other)
                .insert(CaseNoteQuestionAnswersColumn::ValueRadio, option.id)
                .query()
                .execute(database)
                .await?;
        }

        QuestionDataValue::MultiCheckBox { options, other } => {
            let answer = query
                .insert(
                    CaseNoteQuestionAnswersColumn::ResponseType,
                    QuestionType::MultiCheckBox,
                )
                .insert(CaseNoteQuestionAnswersColumn::ValueText, other)
                .return_all()
                .query_as::<CaseNoteQuestionAnswers>()
                .fetch_one(database)
                .await?;

            for option in options {
                answer.add_multi_check_box(option.id, database).await?;
            }
        }
    }
    Ok(())
}

impl DBQuestionResponse {
    pub async fn get_case_note_all(database: &sqlx::PgPool) -> sqlx::Result<Vec<Self>> {
        let query = r#"
        SELECT
            case_note_question_answers.id as answer_id,
            questions.id as question_id,
            questions.string_id as question_string_id,
            questions.string_id_other as question_string_id_other,
            case_note_question_answers.response_type as response_type,
            case_note_question_answers.value_text as value_text,
            case_note_question_answers.value_number as value_number,
            case_note_question_answers.value_float as value_float,

            case_note_question_answers.value_boolean as value_boolean,
            (question_options.id, question_options.name) as value_radio,
            array(
                SELECT (mcb.option_id, qo.name, qo.string_id) FROM case_note_question_answer_mcb as mcb
                                JOIN public.question_options qo on qo.id = mcb.option_id
                                WHERE question_answers_id = case_note_question_answers.id
            ) as options
        FROM case_note_question_answers
            JOIN questions  on case_note_question_answers.question_id = questions.id
            LEFT JOIN question_options on case_note_question_answers.value_radio = question_options.id
        "#;

        let result = sqlx::query_as(query).fetch_all(database).await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {

    use crate::database::red_cap::questions::{CleanQuestionResponse, DBQuestionResponse};

    #[tokio::test]

    pub async fn test() -> anyhow::Result<()> {
        let database = crate::database::tests::setup_query_test().await?;

        let questions = DBQuestionResponse::get_case_note_all(&database).await?;
        let clean_questions = questions
            .into_iter()
            .map(Into::into)
            .collect::<Vec<CleanQuestionResponse>>();
        let json = serde_json::to_string_pretty(&clean_questions)?;
        println!("{}", json);
        Ok(())
    }
}
