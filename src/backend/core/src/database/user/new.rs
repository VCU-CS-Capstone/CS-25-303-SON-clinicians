use crate::database::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use utoipa::ToSchema;

use super::{
    User, UserColumn,
    auth::{UserPasswordAuthentication, UserPasswordAuthenticationColumn},
};

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

    pub async fn insert_return_user(
        self,
        database: impl Executor<'_, Database = sqlx::Postgres>,
    ) -> DBResult<User> {
        let user = InsertQueryBuilder::new(User::table_name())
            .insert(UserColumn::Username, self.username.value())
            .insert(UserColumn::Email, self.email.value())
            .insert(UserColumn::FirstName, self.first_name.value())
            .insert(UserColumn::LastName, self.last_name.value())
            .return_all()
            .query_as()
            .fetch_one(database)
            .await?;
        Ok(user)
    }
}
#[instrument]
pub async fn create_or_update_user_password(
    user_id: i32,
    password: &str,
    database: impl Executor<'_, Database = sqlx::Postgres>,
) -> DBResult<()> {
    InsertQueryBuilder::new(UserPasswordAuthentication::table_name())
        .insert(UserPasswordAuthenticationColumn::UserId, user_id)
        .insert(UserPasswordAuthenticationColumn::Password, password)
        .on_conflict(
            ConflictTarget::Constraint("unique_user_id_password"),
            ConflictActionBuilder::do_update()
                .set_column_to_excluded(UserPasswordAuthenticationColumn::Password)
                .set_column(
                    UserPasswordAuthenticationColumn::UpdatedAt.dyn_column(),
                    DynExpr::new(SqlFunctionBuilder::now()),
                )
                .set_column(
                    UserPasswordAuthenticationColumn::RequiresReset.dyn_column(),
                    DynExpr::new(false),
                ),
        )
        .query()
        .execute(database)
        .await?;
    Ok(())
}
