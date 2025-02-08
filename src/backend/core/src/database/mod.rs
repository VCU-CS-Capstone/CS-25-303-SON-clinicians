mod config;
pub mod red_cap;
pub mod user;
pub use config::*;
pub mod table_utils;
pub mod tools;
use sqlx::{migrate::Migrator, postgres::PgConnectOptions, PgPool};
use tracing::info;
pub mod queries;
/// A bunch of re-exports to make it easier to use the database module.
pub mod prelude {
    pub use super::tools::*;
    pub use super::{DBError, DBResult};
    pub use chrono::{DateTime, FixedOffset, Local, NaiveDate};
    pub use cs25_303_macros::Columns;

    pub use sqlx::{postgres::PgRow, prelude::*, FromRow, PgPool, Postgres, QueryBuilder};
}
pub static MIGRATOR: Migrator = sqlx::migrate!();
#[derive(thiserror::Error, Debug)]
pub enum DBError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error(transparent)]
    Questions(#[from] red_cap::questions::QuestionError),
    #[error("{0}")]
    Other(&'static str),
    #[error("Invalid host must be in the format host:port got `{0}`")]
    InvalidHost(String),
}
/// The type for a DateTime in the database.
///
/// Postgres Type: `TIMESTAMP WITH TIME ZONE`
pub type DBTime = chrono::DateTime<chrono::FixedOffset>;

pub type DBResult<T> = Result<T, DBError>;
pub async fn connect(config: PgConnectOptions, run_migrations: bool) -> Result<PgPool, DBError> {
    let database = PgPool::connect_with(config).await?;
    if run_migrations {
        info!("Running migrations");
        MIGRATOR.run(&database).await?;
        info!("Checking for default questions");
        red_cap::questions::default::add_default_questions(&database).await?;
    }
    Ok(database)
}
