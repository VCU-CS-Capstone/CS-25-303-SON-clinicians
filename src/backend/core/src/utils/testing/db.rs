use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx_postgres::PgConnectOptions;
use tracing::{debug, info};

use crate::database::DBError;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DBTestingConfig {
    pub user: String,
    pub password: String,
    /// If not specified it will create a new database with a random name.
    pub database: Option<String>,
    // The host can be in the format host:port or just host.
    pub host: String,
    // The port is optional. If not specified the default port is used. or will be extracted from the host.
    pub port: Option<u16>,
}
impl DBTestingConfig {
    pub async fn connect(&self) -> Result<PgPool, DBError> {
        // The port can be specified in the host field. If it is, we need to extract it.
        let host = self.host.split(':').collect::<Vec<&str>>();

        let (host, port) = match host.len() {
            // The port is not specified. Use the default port.
            1 => (host[0], self.port.unwrap_or(5432)),
            // The port is specified within the host. The port option is ignored.
            2 => (host[0], host[1].parse::<u16>().unwrap_or(5432)),
            _ => {
                // Not in the format host:port. Possibly IPv6 but we don't support that. As not really supported in the wild.
                return Err(DBError::InvalidHost(self.host.to_owned()));
            }
        };
        let mut options = PgConnectOptions::new()
            .username(&self.user)
            .password(&self.password)
            .host(host)
            .port(port);
        let create_test_db = if let Some(database) = &self.database {
            options = options.database(database);
            false
        } else {
            true
        };

        if !create_test_db {
            return crate::database::connect(options, true).await;
        }

        let database_connection = PgPool::connect_with(options.clone()).await?;

        let mut test_db_number = 1;
        let test_db_name = loop {
            let test_db_name = format!("cs25_303_test_{}", test_db_number);
            if !does_db_exist(&database_connection, &test_db_name).await? {
                break test_db_name;
            }
            debug!(?test_db_name, "Database already exists, trying next number");
            test_db_number += 1;
        };
        info!("Creating test database: {}", test_db_name);
        let query = format!("CREATE DATABASE {}", test_db_name);
        sqlx::query(&query).execute(&database_connection).await?;

        {
            database_connection.close().await;
            drop(database_connection);
        }

        options = options.database(&test_db_name);

        crate::database::connect(options, true).await
    }
}

async fn does_db_exist(pool: &PgPool, db_name: &str) -> Result<bool, DBError> {
    let query = format!(
        "SELECT EXISTS(SELECT 1 from pg_database WHERE datname='{}');",
        db_name
    );
    let row: bool = sqlx::query_scalar(&query).fetch_one(pool).await?;
    Ok(row)
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;
    use tracing::warn;

    use crate::{
        database::{red_cap::Locations, DatabaseConfig},
        utils::testing::config::testing::{get_testing_config, no_testing_config},
    };
    #[ignore = "This is just a test to see if the testing is working"]
    #[tokio::test]
    pub async fn test_the_test_db() -> anyhow::Result<()> {
        let Some(config) = get_testing_config() else {
            no_testing_config()?;
            return Ok(());
        };
        config.init_logger();

        let database = config.database;
        let pool = database.connect().await?;

        let locations = Locations::get_all(&pool).await?;

        assert!(locations.len() > 0);

        Ok(())
    }
    #[ignore = "This is going to delete databases be careful."]
    #[tokio::test]
    pub async fn purge_test_dbs() -> anyhow::Result<()> {
        let Some(config) = get_testing_config() else {
            no_testing_config()?;
            return Ok(());
        };
        config.init_logger();
        let testing_config = config.database;
        let delete_config = DatabaseConfig {
            user: testing_config.user.clone(),
            password: testing_config.password,
            host: testing_config.host,
            port: testing_config.port,
            database: testing_config.user.clone(),
        };
        let database_connection = PgPool::connect_with(delete_config.try_into()?).await?;

        let query = "SELECT datname FROM pg_database WHERE datname LIKE 'cs25_303_test_%'";
        let rows: Vec<String> = sqlx::query_scalar(query)
            .fetch_all(&database_connection)
            .await?;

        for db_name in rows {
            warn!("Dropping database: {}", db_name);
            let query = format!("DROP DATABASE {}", db_name);
            sqlx::query(&query).execute(&database_connection).await?;
        }

        Ok(())
    }
}
