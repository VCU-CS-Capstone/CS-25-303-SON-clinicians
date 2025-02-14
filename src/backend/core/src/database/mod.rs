mod config;
pub mod red_cap;
pub mod user;
use std::ops::Deref;

pub use config::*;
pub mod table_utils;
use derive_more::{From, Into};
use pg_extended_sqlx_queries::PageParams;
use serde::{Deserialize, Serialize};
use sqlx::{migrate::Migrator, postgres::PgConnectOptions, PgPool};
use tracing::info;
use utoipa::{IntoParams, ToSchema};
pub mod queries;
/// A bunch of re-exports to make it easier to use the database module.
pub mod prelude {
    pub use pg_extended_sqlx_queries::prelude::*;

    pub use super::{DBError, DBResult};
    pub use chrono::{DateTime, FixedOffset, Local, NaiveDate};

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

#[derive(Clone, Copy, PartialEq, Eq, From, Into, Deserialize, ToSchema, IntoParams, Debug)]
#[serde(default)]
#[into_params(parameter_in = Query)]
pub struct CSPageParams {
    /// The number of items per page
    #[param(default = 10)]
    pub page_size: i32,
    /// The page number
    #[param(default = 1)]
    pub page_number: i32,
}
impl Default for CSPageParams {
    fn default() -> Self {
        Self {
            page_size: 10,
            page_number: 1,
        }
    }
}
impl From<PageParams> for CSPageParams {
    fn from(params: PageParams) -> Self {
        Self {
            page_size: params.page_size,
            page_number: params.page_number,
        }
    }
}
impl From<CSPageParams> for PageParams {
    fn from(params: CSPageParams) -> Self {
        Self {
            page_size: params.page_size,
            page_number: params.page_number,
        }
    }
}
/// A paginated response
///
/// Includes a total number of pages and the total number of items
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
    /// The number of items per page
    pub total_pages: i32,
    /// The total number of items in the query
    pub total: i64,
    /// The data for the current page
    pub data: Vec<T>,
}
impl<T> PaginatedResponse<T> {
    pub fn create_response(
        data: Vec<T>,
        page_params: &PageParams,
        total: i64,
    ) -> PaginatedResponse<T> {
        let total_pages = (total as f64 / page_params.page_size as f64).ceil() as i32;
        PaginatedResponse {
            total_pages,
            total,
            data,
        }
    }
}

impl<T> Deref for PaginatedResponse<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Default for PaginatedResponse<T> {
    fn default() -> Self {
        Self {
            total_pages: 0,
            total: 0,
            data: Vec::new(),
        }
    }
}
