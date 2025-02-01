use anyhow::Context;
use clap::Parser;
use cs25_303_core::database::DatabaseConfig;
use human_panic::setup_panic;
use random::RandomParticipantsCommand;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt, Layer};
pub mod config;
pub mod random;
use std::path::{Path, PathBuf};
#[derive(Debug, Clone, Parser)]
pub struct CLI {
    #[clap(flatten)]
    pub database: DatabaseConfig,
    #[clap(short, long)]
    pub config_file: Option<PathBuf>,
    #[clap(subcommand)]
    pub command: Commands,
}
#[derive(Debug, Clone, clap::Subcommand)]
pub enum Commands {
    /// Generate a bunch of random participants
    RandomParticipants(RandomParticipantsCommand),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_panic!();

    let cli = CLI::parse();
    load_logging()?;
    let database = cs25_303_core::database::connect(cli.database.try_into()?, true).await?;
    match cli.command {
        Commands::RandomParticipants(command) => {
            command.run(database).await?;
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
