use sqlx::{prelude::FromRow, PgPool};

use crate::database::prelude::*;
pub mod token;
use super::{DBError, User};
#[derive(Debug)]
pub struct UserAndPasswordAuth {
    pub user: User,
    pub password_auth: Option<UserPasswordAuthentication>,
}
/// Table: user_authentication_password
#[derive(Debug, FromRow, Columns)]
pub struct UserPasswordAuthentication {
    pub id: i32,
    pub user_id: i32,
    /// Passwords will be stored with Argon2.
    ///
    /// Null if the user was setup with password login.
    /// But didn't set a password.
    pub password: Option<String>,
    pub requires_reset: bool,
    pub updated_at: Option<DateTime<FixedOffset>>,
    pub created_at: DateTime<FixedOffset>,
}
impl UserPasswordAuthentication {
    pub async fn find_by_user_id(user_id: i32, db: &PgPool) -> DBResult<Option<Self>> {
        sqlx::query_as("SELECT * FROM user_authentication_password WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(db)
            .await
            .map_err(DBError::from)
    }
}
/// Table: user_authentication_saml
///
/// Contains Data for the Users who are authenticated via SAML.
///
/// I don't know how SAML works yet. This is just a template
#[derive(Debug)]
pub struct SamlAuthentication {
    pub id: i32,
    pub user_id: i32,

    pub saml_id: String,
    pub created_on: DateTime<FixedOffset>,
}
