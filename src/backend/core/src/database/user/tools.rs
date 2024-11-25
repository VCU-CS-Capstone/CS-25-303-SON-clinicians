use crate::database::prelude::*;

use super::{User, UserColumn};
pub async fn does_username_exist(username: &str, database: &sqlx::PgPool) -> DBResult<bool> {
    let result: bool = SelectExists::new(User::table_name())
        .where_equals(UserColumn::Username.lower(), username.to_lowercase())
        .query_scalar()
        .fetch_one(database)
        .await?;
    Ok(result)
}

pub async fn does_email_exist(email: &str, database: &sqlx::PgPool) -> DBResult<bool> {
    let result: bool = SelectExists::new(User::table_name())
        .where_equals(UserColumn::Email.lower(), email.to_lowercase())
        .query_scalar()
        .fetch_one(database)
        .await?;
    Ok(result)
}

pub async fn does_user_id_exist(id: i32, database: &sqlx::PgPool) -> DBResult<bool> {
    let result: bool = SelectExists::new(User::table_name())
        .where_equals(UserColumn::Id, id)
        .query_scalar()
        .fetch_one(database)
        .await?;
    Ok(result)
}
