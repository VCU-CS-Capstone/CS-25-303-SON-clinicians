use ahash::{HashMap, HashMapExt};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json};
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
    pub name: String,
    pub program: Programs,
    pub parent_location: Option<i32>,
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
    pub async fn find_by_name(
        name: &str,
        database: &sqlx::PgPool,
    ) -> Result<Option<Locations>, sqlx::Error> {
        let result = sqlx::query_as(
            r#"
            SELECT * FROM locations
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(database)
        .await?;
        Ok(result)
    }

    pub async fn find_children_of(
        parent_id: i32,
        database: &sqlx::PgPool,
    ) -> Result<Vec<Locations>, DBError> {
        SimpleSelectQueryBuilder::new(Locations::table_name(), &LocationsColumn::all())
            .where_equals(LocationsColumn::ParentLocation, parent_id)
            .query_as()
            .fetch_all(database)
            .await
            .map_err(DBError::from)
    }
    pub async fn get_all(database: &sqlx::PgPool) -> Result<Vec<Locations>, DBError> {
        SimpleSelectQueryBuilder::new(Locations::table_name(), &LocationsColumn::all())
            .query_as()
            .fetch_all(database)
            .await
            .map_err(DBError::from)
    }
    pub async fn find_all_in_program(
        program: Programs,
        database: &sqlx::PgPool,
    ) -> Result<Vec<Locations>, DBError> {
        SimpleSelectQueryBuilder::new(Locations::table_name(), &LocationsColumn::all())
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
    /// RWHP Red Cap ID: `rhwp_location_visit`
    /// MHWP Red Cap ID: `mhwp_location_visit`
    /// Petersburg Sub Red Cap ID: `mhwp_location_visit_petersburg`
    pub visit: HashMap<String, i32>,
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
