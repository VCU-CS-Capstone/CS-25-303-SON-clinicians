use std::borrow::Cow;

use chrono::{DateTime, FixedOffset};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use tracing::{debug, error};

use super::{
    new::{NewQuestion, NewQuestionCategory, NewQuestionOptions},
    DBResult,
};
/// Table name: _default_questions
#[derive(Debug, FromRow)]
pub struct DefaultQuestionsTable {
    pub id: i32,
    pub file_name: String,
    pub added_at: DateTime<FixedOffset>,
}
impl DefaultQuestionsTable {
    pub async fn clear(conn: &PgPool) -> DBResult<()> {
        sqlx::query("DELETE FROM _default_questions")
            .execute(conn)
            .await?;
        Ok(())
    }
    pub async fn was_file_added(file_name: &str, conn: &PgPool) -> DBResult<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM _default_questions WHERE file_name = $1")
                .bind(file_name)
                .fetch_one(conn)
                .await?;
        Ok(count > 0)
    }

    pub async fn insert_file(file_name: &str, conn: &PgPool) -> DBResult<()> {
        sqlx::query("INSERT INTO _default_questions (file_name) VALUES ($1)")
            .bind(file_name)
            .execute(conn)
            .await?;
        Ok(())
    }
}
#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/questions"]
struct DefaultQuestionsData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultQuestionWithOptions {
    pub question: NewQuestion,
    pub options: Option<Vec<NewQuestionOptions>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultQuestions {
    pub after: Option<Vec<String>>,
    pub category: NewQuestionCategory,
    #[serde(default)]
    pub questions: Vec<DefaultQuestionWithOptions>,
}
/// Orders the files based on the after field
///
/// Conn is optional for testing purposes
pub async fn get_question_files(
    conn: Option<&PgPool>,
) -> DBResult<Vec<(Cow<'static, str>, DefaultQuestions)>> {
    let mut question_files = Vec::new();
    for question_file in DefaultQuestionsData::iter() {
        if question_file == "README.md" {
            continue;
        }
        if let Some(conn) = &conn {
            if DefaultQuestionsTable::was_file_added(&question_file, conn).await? {
                continue;
            }
        }
        let question = DefaultQuestionsData::get(&question_file).expect("File Should Exist");
        let mut question: DefaultQuestions =
            serde_json::from_slice(&question.data).expect("This is a bug in the code");
        // Remove after options  that already exist in DefaultQuestionsTable
        // Ignored if conn is None
        if let Some(conn) = &conn {
            if let Some(after) = question.after {
                let mut new_after = Vec::new();
                for after in after {
                    if !DefaultQuestionsTable::was_file_added(&after, conn).await? {
                        new_after.push(after);
                    } else {
                        debug!(
                            "Skipping after requirement {} because it has already been added",
                            after
                        );
                    }
                }
                if new_after.is_empty() {
                    question.after = None;
                } else {
                    question.after = Some(new_after);
                }
            }
        }
        question_files.push((question_file, question));
    }
    // QuestionFiles have an optional after field that is a list of file names that should be added before this file.
    // This is to ensure that the files are added in the correct order.
    // So we will sort the files based on the after field.

    question_files.sort_by(|(a_name, a), (b_name, b)| {
        if a.after.is_none() && b.after.is_none() {
            return std::cmp::Ordering::Equal;
        }
        let a_after = a.after.as_deref().unwrap_or(&[]);
        let b_after = b.after.as_deref().unwrap_or(&[]);
        if a_after.iter().any(|a| a == b_name.as_ref()) {
            std::cmp::Ordering::Greater
        } else if b_after.iter().any(|b| b == a_name.as_ref()) {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    });
    Ok(question_files)
}
pub async fn add_default_questions(conn: &PgPool) -> DBResult<()> {
    let question_files = get_question_files(Some(conn)).await?;
    for (question_file, questions) in question_files {
        let DefaultQuestions {
            category,
            questions,
            ..
        } = questions;
        let category = match category.clone().insert_return_category(conn).await {
            Ok(ok) => ok,
            Err(err) => {
                error!(?category, "Error inserting category: {:?}", err);
                return Err(err);
            }
        };
        for question in questions {
            let DefaultQuestionWithOptions { question, options } = question;
            debug!(?question, "Inserting question");
            let question = match question
                .clone()
                .insert_with_category_return_question(category.id, conn)
                .await
            {
                Ok(ok) => ok,
                Err(err) => {
                    println!("Error inserting question: {:?}", question);
                    error!(?question, "Error inserting question: {:?}", err);
                    return Err(err);
                }
            };
            if let Some(options) = options {
                for option in options {
                    option
                        .insert_with_question_return_options(question.id, conn)
                        .await?;
                }
            }
        }
        DefaultQuestionsTable::insert_file(&question_file, conn).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::database::red_cap::questions::{default::DefaultQuestionsTable, QuestionCategory};

    #[tokio::test]
    #[ignore]
    pub async fn refresh_default_questions() -> anyhow::Result<()> {
        let conn = crate::database::tests::setup_query_test().await?;
        DefaultQuestionsTable::clear(&conn).await?;
        QuestionCategory::delete_all(&conn).await?;
        super::add_default_questions(&conn).await?;
        Ok(())
    }

    #[tokio::test]
    pub async fn get_question_file_order() -> anyhow::Result<()> {
        let question_files = super::get_question_files(None).await?;
        for (file, _) in question_files {
            println!("{}", file);
        }
        Ok(())
    }
}
