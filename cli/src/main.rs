pub mod cli_args;
pub mod notifications;
pub mod queue_messages;
pub mod refresh_search_index;
pub mod scraping;

use shared::private_args::Args;
use tracing::info;

use clap::Parser;

use crate::cli_args::{CliArgs, ScrapeSubCommand};
use crate::notifications::entrypoints::{handle_notifications, handle_send_test_notifications};
use crate::refresh_search_index::refresh_search_index;
use crate::scraping::{handle_backfill_impl, handle_scrape_impl};

/// Main entry point.
///
/// # Errors
/// If anything doesn't go as intented.
///
/// # Panics
/// This function panics when:
///     - If it can't create a PGMQ Queue.
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_args = CliArgs::parse();

    shared::configure_tracing(&cli_args.args);
    info!("{:?}", &cli_args);
    if cli_args.command.is_none() {
        return Err("No commands specified. Exiting".into());
    }
    let command = cli_args.command.unwrap();

    info!("Executing command {}", &command);

    match command {
        cli_args::Commands::Scrape { scrape_sub_command } => {
            handle_scrape(scrape_sub_command, cli_args.args).await?;
        }
        cli_args::Commands::TestRetrieval => handle_test_retrieval(cli_args.args).await?,
        cli_args::Commands::SendNotifications => handle_notifications(cli_args.args).await?,
        cli_args::Commands::SendTestNotification { audiobook_id } => {
            handle_send_test_notifications(audiobook_id, cli_args.args).await?;
        }
        cli_args::Commands::RefreshSearchIndex { every_n_seconds } => {
            refresh_search_index(cli_args.args, every_n_seconds).await?;
        }
    }

    Ok(())
}

#[allow(clippy::unused_async)]
async fn handle_scrape(
    scrape_sub_commmand: ScrapeSubCommand,
    args: Args,
) -> Result<(), Box<dyn std::error::Error>> {
    match scrape_sub_commmand {
        ScrapeSubCommand::Latest => handle_scrape_impl(args).await,
        ScrapeSubCommand::Backfill {
            page_start,
            page_end,
        } => handle_backfill_impl(args, page_start, page_end).await,
    }
}

// Function that handles the scraping of audiobook bay.
#[allow(clippy::unused_async)]
async fn handle_test_retrieval(_common_args: Args) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}
