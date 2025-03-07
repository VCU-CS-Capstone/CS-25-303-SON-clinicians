use anyhow::Context;
use clap::Parser;
use config::DataToolConfig;
use cs25_303_core::database::DatabaseConfig;
use human_panic::setup_panic;
use pull::PullParticipant;
use random::RandomParticipantsCommand;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer, filter, layer::SubscriberExt, util::SubscriberInitExt};
pub mod config;
pub mod pull;
pub mod push;
pub mod random;
use std::path::{Path, PathBuf};
#[derive(Debug, Clone, Parser)]
#[command(
    version,
    about = "CS25-30X Data Tools",
    long_about = "Github Repository: https://github.com/VCU-CS-Capstone/CS-25-303-SON-clinicians",
    author
)]
pub struct CLI {
    #[clap(short, long, default_value = "data-tools.toml")]
    pub config: PathBuf,
    #[clap(subcommand)]
    pub command: Commands,
}
#[derive(Debug, Clone, clap::Subcommand)]
pub enum Commands {
    /// Generate a bunch of random participants
    RandomParticipants(RandomParticipantsCommand),
    /// Pull a participant from redcap
    ///
    /// ## WARNING
    /// You must be connected to the VCU VPN to pull from redcap
    PullParticipant(PullParticipant),
    /// Push a participant to redcap
    ///
    /// ## WARNING
    ///
    /// You must be connected to the VCU VPN to push to redcap
    PushParticipant(push::PushParticipant),
    SaveDefaultConfig,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_panic!();

    let cli = CLI::parse();
    load_logging()?;
    let config_file = config::load_config(&cli.config)?;

    match cli.command {
        Commands::RandomParticipants(command) => {
            command.run(config_file).await?;
        }
        Commands::PullParticipant(command) => {
            pull::execute(command, config_file).await?;
        }
        Commands::PushParticipant(command) => {
            push::execute(command, config_file).await?;
        }
        Commands::SaveDefaultConfig => {
            let default_config = DataToolConfig {
                red_cap_token: Some("MY-API-TOKEN".to_string()),
                database: DatabaseConfig::default(),
            };
            let toml = toml::to_string(&default_config)?;
            std::fs::write(&cli.config, toml)?;
            info!("Default config saved to {}", cli.config.display());
        }
    }
    Ok(())
}
fn load_logging() -> anyhow::Result<()> {
    let stdout_log = tracing_subscriber::fmt::layer().pretty();
    tracing_subscriber::registry()
        .with(
            stdout_log.with_filter(
                filter::Targets::new()
                    .with_target("cs25_303_data_tools", LevelFilter::TRACE)
                    .with_target("cs25_303_core", LevelFilter::TRACE)
                    .with_target("sqlx", LevelFilter::WARN),
            ),
        )
        .init();
    Ok(())
}

pub fn does_file_name_start_with(path: impl AsRef<Path>, start: &str) -> anyhow::Result<bool> {
    let file_name = path
        .as_ref()
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("No file name"))?
        .to_str()
        .context("Invalid file name")?;
    Ok(file_name.starts_with(start))
}
