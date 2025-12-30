use std::fmt::Display;

use clap::{Parser, Subcommand};
use shared::private_args::Args;

#[derive(Subcommand, Debug, Clone)]
pub enum ScrapeSubCommand {
    Latest,

    Backfill {
        /// Start page for the backfilling.
        #[arg(long, default_value_t = 1)]
        page_start: u16,

        /// Highest page for the backfilling.
        #[arg(long, default_value_t = 50)]
        page_end: u16,
    },
}

impl Display for ScrapeSubCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrapeSubCommand::Latest => write!(f, "latest"),
            ScrapeSubCommand::Backfill {
                page_start,
                page_end,
            } => write!(f, "backfilling ({page_start}-{page_end})"),
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scrape the latest audiobookbay page.
    Scrape {
        #[command(subcommand)]
        scrape_sub_command: ScrapeSubCommand,
    },
    /// Backfill the DB by scraping audiobookbay.
    // Command for handling notifications.
    SendNotifications,

    // Send test notification. Used for testing the command above
    SendTestNotification {
        audiobook_id: i64,
    },

    RefreshSearchIndex {
        #[arg(long, default_value_t = 14400)] // Every 4 hours
        every_n_seconds: u32,
    },

    TestRetrieval,
}

impl Display for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Commands::Scrape { scrape_sub_command } => write!(f, "scrape {scrape_sub_command}"),
            Commands::TestRetrieval => write!(f, "test-retrieval"),
            Commands::SendNotifications => write!(f, "send-notifications"),
            Commands::SendTestNotification { audiobook_id } => {
                write!(f, "send-test-notifications {audiobook_id}")
            }
            Commands::RefreshSearchIndex { every_n_seconds } => {
                write!(f, "refresh-search-indexes every {every_n_seconds} seconds")
            }
        }
    }
}

#[derive(Parser, Debug)]
pub struct CliArgs {
    #[clap(flatten)]
    pub args: Args,

    #[command(subcommand)]
    pub command: Option<Commands>,
}
