use super::prelude::*;
use auth::UserAndPasswordAuth;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{database::DBResult, user::Scopes};
pub mod auth;
pub mod roles;

pub trait UserType {
    fn get_id(&self) -> i32;
    fn columns() -> Vec<UserColumn> {
        UserColumn::all()
    }
    async fn does_user_have_scope_or_admin(
        &self,
        scope: Scopes,
        database: &sqlx::PgPool,
    ) -> Result<bool, sqlx::Error> {
        self.does_user_have_any_scope(&[Scopes::Admin, scope], database)
            .await
    }

    async fn does_user_have_any_scope(
        &self,
        scope: &[Scopes],
        database: &sqlx::PgPool,
    ) -> Result<bool, sqlx::Error> {
        let result: i64 = sqlx::query_scalar("
            SELECT count(1) from users
                LEFT JOIN user_roles ON user_roles.user_id = users.id
                LEFT JOIN role_permissions ON role_permissions.role_id = user_roles.role_id AND
                        (role_permissions.permission = ANY($1))
                LEFT JOIN user_permissions ON users.id = user_permissions.user_id AND
                        (user_permissions.permission = ANY($1))
                WHERE users.id = $2 AND ((user_permissions.permission = ANY($1)) OR (role_permissions.permission = ANY($1)))
        ")
                    .bind(scope)
                    .bind(self.get_id()).fetch_one(database).await?;
        Ok(result > 0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize, Deserialize, ToSchema, Columns)]
pub struct User {
    /// The ID of the user.
    pub id: i32,
    /// The username of the user.
    pub username: String,
    /// The email of the user.
    pub email: String,
    /// The first name of the user.
    pub first_name: String,
    /// The last name of the user.
    pub last_name: String,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}
#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize, Deserialize, ToSchema)]
pub struct UserPermissions {
    pub id: i32,
    pub user_id: i32,
    pub scope: Scopes,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}
/// Finds a user by their email or username.
///
/// If user is found it will also return the password authentication data if it exists.
pub async fn find_user_by_email_or_username_with_password_auth(
    email_or_username: impl AsRef<str>,
    db: &sqlx::PgPool,
) -> DBResult<Option<UserAndPasswordAuth>> {
    // TODO use a SQL JOIN
    let Some(user) = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users
        WHERE email = $1 OR username = $1
        "#,
    )
    .bind(email_or_username.as_ref())
    .fetch_optional(db)
    .await?
    else {
        return Ok(None);
    };

    let password_auth = auth::UserPasswordAuthentication::find_by_user_id(user.id, db).await?;

    Ok(Some(UserAndPasswordAuth {
        user,
        password_auth,
    }))
}

pub async fn find_user_by_id(id: i32, db: &sqlx::PgPool) -> DBResult<Option<User>> {
    let user = sqlx::query_as::<_, User>(r#"SELECT * FROM users WHERE id = $1"#)
        .bind(id)
        .fetch_optional(db)
        .await?;

    Ok(user)
}
