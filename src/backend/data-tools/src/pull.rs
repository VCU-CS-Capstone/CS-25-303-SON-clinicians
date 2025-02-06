use anyhow::Context;
use clap::Args;
use cs25_303_core::red_cap::{
    api::RedcapClient,
    converter::RedCapConverter,
    tasks::{pull_case_notes, pull_goals, pull_medications, pull_record_base_types},
};

use crate::config::DataToolConfig;

#[derive(Debug, Clone, Args)]
pub struct PullParticipant {
    /// The participant id to pull from redcap
    ///
    /// This is the record id in redcap
    pub participant_id: i32,
}

pub async fn execute(pull: PullParticipant, config: DataToolConfig) -> anyhow::Result<()> {
    let Some(api_key) = config.red_cap_token else {
        anyhow::bail!("No redcap token provided")
    };

    let client = RedcapClient::new(api_key)
        .await
        .context("Failed to connect to the redcap system")?;
    let database = cs25_303_core::database::connect(config.database.try_into()?, true).await?;

    let mut converter: RedCapConverter = RedCapConverter::new(database.clone()).await?;
    println!("Pulling participant {}", pull.participant_id);

    pull_record_base_types(pull.participant_id, &database, &mut converter, &client).await?;
    pull_medications(pull.participant_id, &database, &client).await?;
    pull_goals(pull.participant_id, &database, &client).await?;
    pull_case_notes(pull.participant_id, &database, &mut converter, &client).await?;
    Ok(())
}
