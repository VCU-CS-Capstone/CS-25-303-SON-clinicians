mod config;
pub mod red_cap;
pub mod user;
pub use config::*;
pub mod table_utils;
pub mod tools;
use sqlx::{migrate::Migrator, postgres::PgConnectOptions, PgPool};
use tracing::info;
/// A bunch of re-exports to make it easier to use the database module.
pub mod prelude {
    pub use super::tools::*;
    pub use super::{DBError, DBResult};
    pub use chrono::{DateTime, FixedOffset, NaiveDate};
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
}
/// The type for a DateTime in the database.
///
/// Postgres Type: `TIMESTAMP WITH TIME ZONE`
pub type DBTime = chrono::DateTime<chrono::FixedOffset>;

pub type DBResult<T> = Result<T, DBError>;
pub async fn connect(config: PgConnectOptions, run_migrations: bool) -> Result<PgPool, DBError> {
    let database = PgPool::connect_with(config).await?;
    // TODO: Add Migration code here
    if run_migrations {
        info!("Running migrations");
        MIGRATOR.run(&database).await?;
        red_cap::questions::default::add_default_questions(&database).await?;
    }
    Ok(database)
}

#[cfg(test)]
pub mod tests {
    use sqlx::PgPool;

    use crate::database::{DatabaseConfig, MIGRATOR};

    #[tokio::test]
    #[ignore = "This test needs a database to run"]
    pub async fn run_migrations() -> anyhow::Result<()> {
        let test_env = crate::env_utils::read_env_file_in_core("test.env")?;

        let config: DatabaseConfig =
            serde_env::from_iter_with_prefix(test_env.iter(), "MIGRATIONS")?;
        let database = PgPool::connect_with(config.try_into()?).await?;

        MIGRATOR.run(&database).await?;

        Ok(())
    }
    pub async fn setup_query_test() -> anyhow::Result<PgPool> {
        crate::test_utils::init_logger();

        let test_env = crate::env_utils::read_env_file_in_core("test.env")?;

        let config: DatabaseConfig = serde_env::from_iter_with_prefix(test_env.iter(), "QUERY")?;
        let database = PgPool::connect_with(config.try_into()?).await?;
        Ok(database)
    }

    pub async fn setup_red_cap_db_test(env: &crate::env_utils::EnvMap) -> anyhow::Result<PgPool> {
        let config: DatabaseConfig = serde_env::from_iter_with_prefix(env.iter(), "RED_CAP")?;
        let database = super::connect(config.try_into()?, true).await?;

        Ok(database)
    }
}
