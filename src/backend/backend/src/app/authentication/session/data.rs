use chrono::{DateTime, Duration, FixedOffset, Local};
use cs25_303_core::database::{self, DBError, user::User};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use tracing::error;
use utoipa::ToSchema;

use super::SessionError;
pub type SessionTime = DateTime<FixedOffset>;
/// A session type.
/// Stored in the session manager.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, ToSchema)]
pub struct Session {
    pub user_id: i32,
    pub session_key: String,
    pub login_id: Uuid,
    pub expires: DateTime<FixedOffset>,
    pub created: DateTime<FixedOffset>,
}
pub type SessionTuple<'value> = (i32, &'value str, &'value [u8; 16], String, String);

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, ToSchema)]
pub struct SessionPublic {
    /// The user id of the session
    pub user_id: i32,
    /// The session key. This is sent to the server to identify the client
    pub session_key: String,
    /// When the session expires
    pub expires: DateTime<FixedOffset>,
    /// When the session was created
    pub created: DateTime<FixedOffset>,
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, ToSchema)]
pub struct SmallSession {
    pub user_id: i32,
    pub session_key: String,
}
impl TryFrom<SessionTuple<'_>> for SmallSession {
    type Error = SessionError;
    fn try_from(tuple: SessionTuple) -> Result<Self, Self::Error> {
        let (user_id, session_key, _, _, _) = tuple;
        Ok(Self {
            user_id,
            session_key: session_key.to_owned(),
        })
    }
}
impl TryFrom<SessionTuple<'_>> for SessionPublic {
    type Error = SessionError;
    fn try_from(tuple: SessionTuple) -> Result<Self, Self::Error> {
        let (user_id, session_key, _, expires, created) = tuple;
        Ok(Self {
            user_id,
            session_key: session_key.to_owned(),
            expires: from_timestamp(&expires, "expires")?,
            created: from_timestamp(&created, "created")?,
        })
    }
}

impl TryFrom<SessionTuple<'_>> for Session {
    type Error = SessionError;
    fn try_from(tuple: SessionTuple) -> Result<Self, Self::Error> {
        let (user_id, session_key, foot_print, expires, created) = tuple;
        let result = Self {
            user_id,
            session_key: session_key.to_owned(),
            login_id: Uuid::from_bytes(*foot_print),
            expires: from_timestamp(&expires, "expires")?,
            created: from_timestamp(&created, "created")?,
        };
        Ok(result)
    }
}
impl Session {
    /// Checks if the session is expired.
    pub fn is_expired(&self) -> bool {
        self.expires < Local::now().fixed_offset()
    }

    pub async fn get_user(&self, db: &sqlx::PgPool) -> Result<Option<User>, DBError> {
        database::user::find_user_by_id(self.user_id, db).await
    }
}
/// A tuple of (user_id, session_id, expires, created)
impl Session {
    pub fn new(user_id: i32, session_id: String, login_id: Uuid, life: Duration) -> Self {
        Self {
            user_id,
            session_key: session_id,
            login_id,
            expires: Local::now().fixed_offset() + life,
            created: Local::now().fixed_offset(),
        }
    }

    pub fn as_tuple_ref(&self) -> SessionTuple {
        (
            self.user_id,
            self.session_key.as_str(),
            self.login_id.as_bytes(),
            self.expires.to_rfc3339(),
            self.created.to_rfc3339(),
        )
    }
}

fn from_timestamp(raw: &str, timestamp_name: &'static str) -> Result<SessionTime, SessionError> {
    DateTime::<FixedOffset>::parse_from_rfc3339(raw).map_err(|err| {
        error!(
            "Failed to parse {}. Delete the Sessions Database: {:?}",
            timestamp_name, err
        );
        SessionError::DateTimeParseError(err)
    })
}
