use std::{
    fmt::Debug,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration as StdDuration,
};

use axum::response::{IntoResponse, Response};
use chrono::{Duration, Local};
use http::StatusCode;
use rand::{distributions::Alphanumeric, rngs::StdRng, Rng, SeedableRng};
use redb::{CommitError, Database, Error, ReadableTable, ReadableTableMetadata, TableDefinition};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, instrument};
use tuxs_config_types::chrono_types::duration::ConfigDuration;
mod data;
use crate::{
    app::{error::IntoErrorResponse, SiteStateInner},
    config::Mode,
};
pub use data::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoginMethod {
    Password,
}
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session not found")]
    RedbError(#[from] redb::Error),
    #[error(transparent)]
    TableError(#[from] redb::TableError),
    #[error(transparent)]
    TransactionError(#[from] redb::TransactionError),
    #[error(transparent)]
    StorageError(#[from] redb::StorageError),
    #[error(transparent)]
    CommitError(#[from] CommitError),
    #[error("Could not parse DateTime: {0}")]
    DateTimeParseError(#[from] chrono::ParseError),
}
impl IntoResponse for SessionError {
    fn into_response(self) -> axum::response::Response {
        error!("{}", self);
        let message = format!(
            "Session Manager Error {:?}. Please Contact the Admin about Session DB Corruption",
            self
        );
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(message.into())
            .unwrap()
    }
}
impl IntoErrorResponse for SessionError {
    fn into_response_boxed(self: Box<Self>) -> axum::response::Response {
        (*self).into_response()
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SessionManagerConfig {
    pub lifespan: ConfigDuration,
    pub cleanup_interval: ConfigDuration,
    pub database_location: PathBuf,
}
impl Default for SessionManagerConfig {
    fn default() -> Self {
        Self {
            lifespan: Duration::days(1).into(),
            cleanup_interval: Duration::hours(1).into(),
            database_location: PathBuf::from("sessions.db"),
        }
    }
}

const TABLE: TableDefinition<&str, SessionTuple> = TableDefinition::new("sessions");

pub struct SessionManager {
    config: SessionManagerConfig,
    sessions: Database,
    mode: Mode,
    running: AtomicBool,
}
impl Debug for SessionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionManager")
            .field("config", &self.config)
            .field("number_of_sessions", &self.number_of_sessions().ok())
            .field("mode", &self.mode)
            .field("running", &self.running.load(Ordering::Relaxed))
            .finish()
    }
}
impl SessionManager {
    pub fn new(session_config: Option<SessionManagerConfig>, mode: Mode) -> Result<Self, Error> {
        let session_config = session_config.unwrap_or_default();
        let sessions = if session_config.database_location.exists() {
            let database = Database::open(&session_config.database_location)?;
            if mode == Mode::Debug {
                println!("Opened database: {:?}", database);
                let session = database.begin_write()?;
                let table = session.open_table(TABLE)?;
                debug!("Found {} sessions", table.len()?);
            }
            database
        } else {
            Database::create(&session_config.database_location)?
        };

        Ok(Self {
            config: session_config,
            sessions,
            mode,
            running: AtomicBool::new(false),
        })
    }
    pub fn number_of_sessions(&self) -> Result<u64, SessionError> {
        let sessions = self.sessions.begin_read()?;
        let table = sessions.open_table(TABLE)?;
        let len = table.len()?;
        Ok(len)
    }
    pub fn filter_table<F>(
        &self,
        continue_on_err: bool,
        filter: F,
    ) -> Result<Vec<Session>, SessionError>
    where
        F: Fn(&Session) -> bool,
    {
        let sessions = self.sessions.begin_read()?;
        let table = sessions.open_table(TABLE)?;
        let mut sessions = Vec::new();
        for index in table.iter()? {
            let value = match index {
                Ok((_, value)) => value,
                Err(err) => {
                    error!("Failed to iterate over sessions: {:?}", err);
                    if !continue_on_err {
                        return Err(err.into());
                    }
                    continue;
                }
            };
            let session = match Session::from_tuple(value.value()) {
                Ok(ok) => ok,
                Err(err) => {
                    error!("Failed to parse session: {:?}", err);
                    if !continue_on_err {
                        return Err(err);
                    }
                    continue;
                }
            };
            if filter(&session) {
                sessions.push(session);
            }
        }
        Ok(sessions)
    }
    #[instrument]
    pub fn clean_inner(&self) -> Result<u32, SessionError> {
        let mut sessions_removed = 0u32;
        let now = Local::now();
        let to_remove = self.filter_table(true, |session| session.expires < now)?;
        if self.mode.is_debug() {
            debug!(?to_remove, "Sessions to remove");
        }
        let sessions = self.sessions.begin_write()?;
        {
            let mut table = sessions.open_table(TABLE)?;
            for key in to_remove {
                debug!("Removing session: {:?}", key);
                match table.remove(&*key.session_id) {
                    Ok(ok) => {
                        if self.mode == Mode::Debug {
                            let ok = ok.map(|x| Session::from_tuple(x.value()));
                            debug!("Removed session: {:?}", ok);
                        }
                        sessions_removed += 1;
                    }
                    Err(err) => {
                        error!("Failed to remove session: {:?}", err);
                    }
                }
            }
        }
        sessions.commit()?;
        Ok(sessions_removed)
    }
    async fn cleaner_task(this: Arc<SiteStateInner>, how_often: StdDuration) {
        let session_manager = &this.session;

        while session_manager.running.load(Ordering::Relaxed) {
            info!("Cleaning sessions");
            let sleep_for = match session_manager.clean_inner() {
                Ok(value) => {
                    info!("Cleaned {} sessions", value);
                    how_often
                }
                Err(err) => {
                    error!("Failed to clean sessions: {:?}", err);
                    how_often / 2
                }
            };
            tokio::time::sleep(sleep_for).await
        }
    }
    pub fn stop_cleaner(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
    pub fn start_cleaner(this: Arc<SiteStateInner>) -> Option<JoinHandle<()>> {
        let how_often = match this.session.config.cleanup_interval.to_std() {
            Ok(ok) => ok,
            Err(err) => {
                error!("Failed to convert cleanup interval: {:?}", err);
                return None;
            }
        };
        this.session.running.store(true, Ordering::Relaxed);
        debug!("Starting Session Cleaner with interval: {:?}", how_often);
        let result = tokio::spawn(async move {
            let this = this;
            SessionManager::cleaner_task(this, how_often).await;
        });
        Some(result)
    }
    #[instrument]
    pub fn create_session(
        &self,
        user_id: i32,
        user_agent: String,
        ip_address: String,
        life: Duration,
    ) -> Result<Session, SessionError> {
        let sessions = self.sessions.begin_write()?;
        let mut session_table = sessions.open_table(TABLE)?;

        let session_id =
            create_session_id(|x| session_table.get(x).map(|x| x.is_some()).unwrap_or(false));
        let session = Session::new(user_id, session_id.clone(), user_agent, ip_address, life);

        session_table.insert(&*session_id, session.as_tuple_ref())?;
        drop(session_table);
        sessions.commit()?;
        Ok(session)
    }
    #[instrument]
    pub fn create_session_default_lifespan(
        &self,
        user_id: i32,
        user_agent: String,
        ip_address: String,
    ) -> Result<Session, SessionError> {
        self.create_session(
            user_id,
            user_agent,
            ip_address,
            self.config.lifespan.duration,
        )
    }
    #[instrument]
    pub fn get_session(&self, session_id: &str) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.begin_read()?;

        let session = sessions.open_table(TABLE)?;
        let session = session
            .get(session_id)?
            .map(|x| Session::from_tuple(x.value()))
            .transpose()?;
        Ok(session)
    }
    #[instrument]
    pub fn delete_session(&self, session_id: &str) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.begin_write()?;
        let mut table = sessions.open_table(TABLE)?;
        let session = table
            .remove(session_id)?
            .map(|x| Session::from_tuple(x.value()))
            .transpose()?;
        drop(table);
        sessions.commit()?;
        Ok(session)
    }
    pub fn delete_all_for_user(&self, user_id: i32) -> Result<(), SessionError> {
        let to_remove = self.filter_table(true, |session| session.user_id == user_id)?;
        let sessions = self.sessions.begin_write()?;
        {
            let mut table = sessions.open_table(TABLE)?;
            for key in to_remove {
                debug!("Removing session: {:?}", key);
                match table.remove(&*key.session_id) {
                    Ok(ok) => {
                        if self.mode == Mode::Debug {
                            let ok = ok.map(|x| Session::from_tuple(x.value()));
                            debug!("Removed session: {:?}", ok);
                        }
                    }
                    Err(err) => {
                        error!("Failed to remove session: {:?}", err);
                    }
                }
            }
        }
        sessions.commit()?;
        Ok(())
    }
}

#[inline(always)]
pub fn create_session_id(exists_call_back: impl Fn(&str) -> bool) -> String {
    let mut rand = StdRng::from_entropy();
    loop {
        let session_id: String = (0..7).map(|_| rand.sample(Alphanumeric) as char).collect();
        if !exists_call_back(&session_id) {
            break session_id;
        }
    }
}
