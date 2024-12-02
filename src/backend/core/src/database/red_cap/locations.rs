use ahash::{HashMap, HashMapExt};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json};
use tracing::instrument;
use utoipa::ToSchema;

use crate::{
    database::prelude::*,
    red_cap::Programs,
    red_cap::{RedCapDataSet, RedCapType},
};

/// Table Name: locations
///
/// This is the table of locations that are used in the system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, Columns, ToSchema)]
pub struct Locations {
    pub id: i32,
    /// Location Name
    pub name: String,
    /// The Program the Location is in
    pub program: Programs,
    /// The parent location
    pub parent_location: Option<i32>,
    /// Rules for connecting to Red Cap
    #[schema(value_type = RedCapLocationConnectionRules)]
    pub red_cap_connection_rules: Json<RedCapLocationConnectionRules>,
}
impl TableType for Locations {
    type Columns = LocationsColumn;
    fn table_name() -> &'static str {
        "locations"
    }
}
impl Locations {
    /// Find a location by its name
    ///
    /// This is used to find the location by its name.
    #[instrument]
    pub async fn find_by_name(
        name: &str,
        database: &sqlx::PgPool,
    ) -> Result<Option<Locations>, sqlx::Error> {
        let result =
            SimpleSelectQueryBuilderV2::new(Locations::table_name(), LocationsColumn::all())
                .where_equals(LocationsColumn::Name, name)
                .query_as()
                .fetch_optional(database)
                .await?;
        Ok(result)
    }
    /// Find all children locations of a parent location
    #[instrument]
    #[inline]
    pub async fn find_children(&self, database: &sqlx::PgPool) -> Result<Vec<Locations>, DBError> {
        Self::find_children_of(self.id, database).await
    }
    /// Find all children locations of a parent location
    #[instrument]
    pub async fn find_children_of(
        parent_id: i32,
        database: &sqlx::PgPool,
    ) -> Result<Vec<Locations>, DBError> {
        SimpleSelectQueryBuilderV2::new(Locations::table_name(), LocationsColumn::all())
            .where_equals(LocationsColumn::ParentLocation, parent_id)
            .query_as()
            .fetch_all(database)
            .await
            .map_err(DBError::from)
    }
    /// Get all locations in the system
    #[instrument]
    pub async fn get_all(database: &sqlx::PgPool) -> Result<Vec<Locations>, DBError> {
        SimpleSelectQueryBuilderV2::new(Locations::table_name(), LocationsColumn::all())
            .query_as()
            .fetch_all(database)
            .await
            .map_err(DBError::from)
    }
    /// Find all locations in a program
    #[instrument]
    pub async fn find_all_in_program(
        program: Programs,
        database: &sqlx::PgPool,
    ) -> Result<Vec<Locations>, DBError> {
        SimpleSelectQueryBuilderV2::new(Locations::table_name(), LocationsColumn::all())
            .where_equals(LocationsColumn::Program, program)
            .query_as()
            .fetch_all(database)
            .await
            .map_err(DBError::from)
    }
}
/// So In Red Cap locations are split over multiple questions.
///
/// This is my easy way to convert the Red Cap locations into a single location.
///
/// This will also leave the door open for more locations to be added in the future.
///
/// Each field corresponds to the field name in Red Cap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, ToSchema)]
pub struct RedCapLocationConnectionRules {
    /// Visit Rules to find
    /// RWHP Red Cap ID: `rhwp_location_visit`
    /// MHWP Red Cap ID: `mhwp_location_visit`
    /// Petersburg Sub Red Cap ID: `mhwp_location_visit_petersburg`
    pub visit: HashMap<String, i32>,
    /// Participant Rules to find
    ///
    /// Used when reading the data from Red Cap
    pub participant: HashMap<String, i32>,
}
impl RedCapLocationConnectionRules {
    pub fn does_match_participant(&self, location: &RedCapLocationRules) -> bool {
        self.participant == location.rules
    }
    pub fn does_match_visit(&self, location: &RedCapLocationRules) -> bool {
        self.visit == location.rules
    }
    pub fn participant_rules(&self) -> RedCapLocationRules {
        RedCapLocationRules {
            rules: self.participant.clone(),
        }
    }
    pub fn visit_rules(&self) -> RedCapLocationRules {
        RedCapLocationRules {
            rules: self.visit.clone(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedCapLocationRules {
    pub rules: HashMap<String, i32>,
}

impl RedCapType for RedCapLocationRules {
    fn read<D: RedCapDataSet>(data: &D) -> Option<Self>
    where
        Self: Sized,
    {
        let mut rules = HashMap::new();
        for (key, value) in data.iter() {
            if key.starts_with("rhwp_location") {
                let value = value.to_number();
                if let Some(value) = value {
                    rules.insert(key.clone(), value as i32);
                }
            }
            if key.starts_with("mhwp_location") {
                let value = value.to_number();
                if let Some(value) = value {
                    rules.insert(key.clone(), value as i32);
                }
            }
        }

        Some(Self { rules })
    }

    fn write<D: RedCapDataSet>(&self, data: &mut D) {
        for (key, value) in &self.rules {
            data.insert(key, (*value).into());
        }
    }
}
