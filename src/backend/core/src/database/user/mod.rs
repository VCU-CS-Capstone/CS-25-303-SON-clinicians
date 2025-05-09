use std::{fmt::Debug, future::Future};

use super::{PaginatedResponse, prelude::*};
use auth::UserAndPasswordAuth;
use pg_extended_sqlx_queries::pagination::PaginationSupportingTool;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use utoipa::ToSchema;

use crate::{database::DBResult, user::Permissions};
pub mod auth;
pub mod new;
pub mod roles;
mod tools;
pub use tools::*;
pub mod login;
pub trait UserType: for<'r> FromRow<'r, PgRow> + Unpin + Send + Sync + Debug + TableQuery {
    fn get_id(&self) -> i32;
    async fn does_user_have_scope_or_admin(
        &self,
        scope: Permissions,
        database: &sqlx::PgPool,
    ) -> Result<bool, sqlx::Error> {
        self.does_user_have_any_scope(&[Permissions::Admin, scope], database)
            .await
    }
    async fn get_by_id(id: i32, database: &sqlx::PgPool) -> DBResult<Option<Self>>
    where
        Self: Sized,
    {
        let result = SelectQueryBuilder::with_columns(User::table_name(), Self::columns())
            .filter(UserColumn::Id.equals(id))
            .query_as()
            .fetch_optional(database)
            .await?;
        Ok(result)
    }

    async fn does_user_have_any_scope(
        &self,
        scope: &[Permissions],
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
    #[instrument]
    fn has_permission(
        &self,
        permission: Permissions,
        database: &PgPool,
    ) -> impl Future<Output = Result<bool, DBError>> + Send {
        async { Ok(true) }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize, Deserialize, ToSchema, TableType)]
#[table(name = "users")]
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
    pub updated_at: DateTime<FixedOffset>,
    pub created_at: DateTime<FixedOffset>,
}
impl UserType for User {
    fn get_id(&self) -> i32 {
        self.id
    }
}

impl User {
    pub async fn get_all_paginated(
        database: &sqlx::PgPool,
        page_size: i32,
        page: i32,
    ) -> DBResult<PaginatedResponse<User>> {
        let page = page - 1;
        let mut query = SelectQueryBuilder::with_columns(User::table_name(), User::columns());
        if page_size > 0 {
            query.limit(page_size);
        }
        if page > 0 {
            query.offset(page * page_size);
        }
        let result = query.query_as().fetch_all(database).await?;

        let result = PaginatedResponse {
            data: result,
            ..Default::default()
        };

        Ok(result)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize, Deserialize, ToSchema)]
pub struct UserPermissions {
    pub id: i32,
    pub user_id: i32,
    pub scope: Permissions,
    pub created_at: DateTime<FixedOffset>,
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
