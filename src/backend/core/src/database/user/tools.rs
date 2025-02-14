use crate::database::prelude::*;

use super::{User, UserColumn};
pub async fn does_username_exist(username: &str, database: &sqlx::PgPool) -> DBResult<bool> {
    let result: bool = SelectExists::new(User::table_name())
        .filter(
            UserColumn::Username
                .lower()
                .equals(username.to_lowercase().value()),
        )
        .query_scalar()
        .fetch_one(database)
        .await?;
    Ok(result)
}

pub async fn does_email_exist(email: &str, database: &sqlx::PgPool) -> DBResult<bool> {
    let result: bool = SelectExists::new(User::table_name())
        .filter(
            UserColumn::Email
                .lower()
                .equals(email.to_lowercase().value()),
        )
        .query_scalar()
        .fetch_one(database)
        .await?;
    Ok(result)
}

pub async fn does_user_id_exist(id: i32, database: &sqlx::PgPool) -> DBResult<bool> {
    let result: bool = SelectExists::new(User::table_name())
        .filter(UserColumn::Id.equals(id.value()))
        .query_scalar()
        .fetch_one(database)
        .await?;
    Ok(result)
}
