use clap::Args;
use cs25_303_core::{
    database::red_cap::participants::Participants,
    red_cap::{api::RedcapClient, converter::RedCapConverter, tasks::push::*},
};

use crate::config::DataToolConfig;

#[derive(Debug, Clone, Args)]
pub struct PushParticipant {
    /// The participant id to pull from redcap
    pub participant_id: Option<i32>,
    #[clap(long)]
    pub all: bool,
}

pub async fn execute(pull: PushParticipant, config: DataToolConfig) -> anyhow::Result<()> {
    let Some(api_key) = config.red_cap_token else {
        anyhow::bail!("No redcap token provided")
    };

    let client = RedcapClient::new(api_key).await?;
    let database = cs25_303_core::database::connect(config.database.try_into()?, true).await?;

    let mut converter: RedCapConverter = RedCapConverter::new(database.clone()).await?;
    if let Some(participant_id) = pull.participant_id {
        push_participant_to_red_cap(participant_id, &database, &mut converter, &client).await?;
        push_participant_medications_to_red_cap(participant_id, &database, &client).await?;
        push_participant_goals_to_red_cap(participant_id, &database, &mut converter, &client)
            .await?;
        push_case_notes_to_redcap(participant_id, &database, &mut converter, &client).await?;
        return Ok(());
    } else if pull.all {
        let participants = Participants::get_all_ids(&database).await?;
        for participant_id in participants {
            println!("Pushing participant {}", participant_id);
            push_participant_to_red_cap(participant_id, &database, &mut converter, &client).await?;
            push_participant_medications_to_red_cap(participant_id, &database, &client).await?;
            push_participant_goals_to_red_cap(participant_id, &database, &mut converter, &client)
                .await?;
            push_case_notes_to_redcap(participant_id, &database, &mut converter, &client).await?;
        }
        return Ok(());
    } else {
        anyhow::bail!("You must provide either a participant id or the --all flag");
    }
}
