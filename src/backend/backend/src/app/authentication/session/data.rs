use chrono::{DateTime, Duration, FixedOffset, Local};
use cs25_303_core::database::{self, user::User, DBError};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use super::SessionError;
pub type SessionTime = DateTime<FixedOffset>;
/// A session type.
/// Stored in the session manager.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, ToSchema)]
pub struct Session {
    pub user_id: i32,
    pub session_id: String,
    pub user_agent: String,
    pub ip_address: String,
    pub expires: DateTime<FixedOffset>,
    pub created: DateTime<FixedOffset>,
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, ToSchema)]
pub struct SmallSession {
    pub user_id: i32,
    pub session_id: String,
}
impl From<SessionTuple<'_>> for SmallSession {
    fn from(tuple: SessionTuple) -> Self {
        let (user_id, session_id, _, _, _, _) = tuple;
        Self {
            user_id,
            session_id: session_id.to_owned(),
        }
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
pub type SessionTuple<'value> = (i32, &'value str, &'value str, &'value str, String, String);
impl Session {
    pub fn new(
        user_id: i32,
        session_id: String,
        user_agent: String,
        ip_address: String,
        life: Duration,
    ) -> Self {
        Self {
            user_id,
            session_id,
            user_agent,
            ip_address,
            expires: Local::now().fixed_offset() + life,
            created: Local::now().fixed_offset(),
        }
    }
    pub fn from_tuple(tuple: SessionTuple) -> Result<Self, SessionError> {
        let (user_id, session_id, user_agent, ip_addr, expires, created) = tuple;

        let session = Session {
            user_id,
            session_id: session_id.to_owned(),
            user_agent: user_agent.to_owned(),
            ip_address: ip_addr.to_owned(),
            expires: from_timestamp(&expires, "expires")?,
            created: from_timestamp(&created, "created")?,
        };
        Ok(session)
    }
    pub fn as_tuple_ref(&self) -> SessionTuple {
        (
            self.user_id,
            self.session_id.as_str(),
            self.user_agent.as_str(),
            self.ip_address.as_str(),
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
