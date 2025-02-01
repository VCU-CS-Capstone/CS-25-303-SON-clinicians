use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use rust_embed::Embed;
use serde::de::DeserializeOwned;
use tracing::{debug, info};

use super::set::RandomSets;

#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/random"]
struct RandomData;
impl RandomData {}
/// Loads a set file from
#[tracing::instrument]
pub fn load_random_set<T, P>(name: &str, sets_override: Option<P>) -> anyhow::Result<T>
where
    T: DeserializeOwned,
    P: AsRef<Path> + Debug,
{
    let file_name = format!("{}.json", name);
    if let Some(sets_override) = sets_override {
        let path = sets_override.as_ref().join(&file_name);
        if path.exists() {
            info!("Loading random set from path: {:?}", path);
            let content = std::fs::File::open(&path)?;
            let result = serde_json::from_reader(&content)?;
            return Ok(result);
        } else {
            debug!("No file found at {:?}", path);
        }
    }
    debug!("Loading random set from embeded data");
    let data = RandomData::get(&format!("sets/{}", file_name))
        .ok_or_else(|| anyhow::anyhow!("No random set found"))?
        .data;

    let random_sets: T = serde_json::from_slice(&data)?;
    Ok(random_sets)
}

pub fn load_random_sets(path_overide: Option<PathBuf>) -> anyhow::Result<RandomSets> {
    let result = RandomSets {
        participants: load_random_set("participants", path_overide.as_ref())?,
        goals: load_random_set("goals", path_overide.as_ref())?,
        medications: load_random_set("medications", path_overide.as_ref())?,
        behbehavioral_risks_identified: load_random_set(
            "behavioral_risks_identified",
            path_overide.as_ref(),
        )?,
        reasons_for_visit: load_random_set("reason_for_visit", path_overide.as_ref())?,
        info_provided_by_caregiver: load_random_set("info_by_caregiver", path_overide.as_ref())?,
        case_note_other_health_measures: load_random_set(
            "case_note_other_health_measures",
            path_overide.as_ref(),
        )?,
        ..Default::default()
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::{env, path::PathBuf};

    use crate::config::testing::{get_testing_config, no_db_connection, no_testing_config};

    #[test]
    pub fn load_full() -> anyhow::Result<()> {
        let random_sets = super::load_random_sets(None)?;
        validate_data_sets(&random_sets);
        Ok(())
    }

    #[test]
    pub fn load_full_with_path() -> anyhow::Result<()> {
        let local_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("random")
            .join("sets");
        assert!(local_path.exists());

        let random_sets = super::load_random_sets(Some(local_path))?;
        validate_data_sets(&random_sets);
        Ok(())
    }
    #[tokio::test]
    pub async fn load_with_config() -> anyhow::Result<()> {
        let Some(test) = get_testing_config()? else {
            no_testing_config()?;
            return Ok(());
        };
        let mut random_sets = super::load_random_sets(None)?;

        if let Some(database) = test.database {
            let database = database.connect().await?;

            random_sets.load_locations(&database).await?;

            assert!(random_sets.m_locations.len() > 0);
            assert!(random_sets.r_locations.len() > 0);
        } else {
            no_db_connection()?;
        }
        validate_data_sets(&random_sets);
        Ok(())
    }
    fn validate_data_sets(random_sets: &super::RandomSets) {
        assert!(random_sets.participants.len() > 0);
        assert!(random_sets.goals.len() > 0);
        assert!(random_sets.medications.len() > 0);
        assert!(random_sets.behbehavioral_risks_identified.len() > 0);
        assert!(random_sets.reasons_for_visit.len() > 0);
        assert!(random_sets.info_provided_by_caregiver.len() > 0);
        assert!(random_sets.case_note_other_health_measures.len() > 0);
    }
}
