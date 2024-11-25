use crate::database::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{User, UserColumn};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NewUser {
    /// The username of the user.
    pub username: String,
    /// The email of the user.
    pub email: String,
    /// The first name of the user.
    pub first_name: String,
    /// The last name of the user.
    pub last_name: String,
}

impl NewUser {
    #[inline]
    pub async fn check_if_email_is_in_use(&self, database: &sqlx::PgPool) -> DBResult<bool> {
        super::does_email_exist(&self.email, database).await
    }
    #[inline]
    pub async fn check_if_username_is_in_use(&self, database: &sqlx::PgPool) -> DBResult<bool> {
        super::does_username_exist(&self.username, database).await
    }

    pub async fn insert_return_user(self, database: &sqlx::PgPool) -> DBResult<User> {
        let user = SimpleInsertQueryBuilder::new(User::table_name())
            .insert(UserColumn::Username, self.username)
            .insert(UserColumn::Email, self.email)
            .insert(UserColumn::FirstName, self.first_name)
            .insert(UserColumn::LastName, self.last_name)
            .return_all()
            .query_as()
            .fetch_one(database)
            .await?;
        Ok(user)
    }
}
