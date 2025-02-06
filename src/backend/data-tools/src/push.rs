use clap::Args;
use cs25_303_core::red_cap::{api::RedcapClient, converter::RedCapConverter, tasks::push::*};

use crate::config::DataToolConfig;

#[derive(Debug, Clone, Args)]
pub struct PushParticipant {
    /// The participant id to pull from redcap
    pub participant_id: i32,
}

pub async fn execute(pull: PushParticipant, config: DataToolConfig) -> anyhow::Result<()> {
    let Some(api_key) = config.red_cap_token else {
        anyhow::bail!("No redcap token provided")
    };

    let client = RedcapClient::new(api_key).await?;
    let database = cs25_303_core::database::connect(config.database.try_into()?, true).await?;

    let mut converter: RedCapConverter = RedCapConverter::new(database.clone()).await?;
    println!("Pulling participant {}", pull.participant_id);

    push_participant_to_red_cap(pull.participant_id, &database, &mut converter, &client).await?;
    push_participant_medications_to_red_cap(pull.participant_id, &database, &client).await?;
    push_participant_goals_to_red_cap(pull.participant_id, &database, &mut converter, &client)
        .await?;
    push_case_notes_to_redcap(pull.participant_id, &database, &mut converter, &client).await?;
    Ok(())
}
