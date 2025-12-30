use clap;
use clap::Parser;
use entities_lib::entities::shareable_args::ShareableArgsValues;

#[derive(Debug, Parser, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Username for postgres.
    #[arg(long, default_value_t=String::from("postgres"))]
    pub postgres_username: String,

    /// Username for postgres.
    #[arg(long, default_value_t=String::from("password"))]
    pub postgres_password: String,

    /// URL for postgres.
    #[arg(long, default_value_t=String::from("127.0.0.1:5432"))]
    pub postgres_url: String,

    /// Username for pgmq.
    #[arg(long, default_value_t=String::from("postgres"))]
    pub pgmq_username: String,

    /// Username for pgmq.
    #[arg(long, default_value_t=String::from("password"))]
    pub pgmq_password: String,

    #[arg(long, default_value_t=String::from("postgres"))]
    pub pgmq_database: String,

    #[arg(long, default_value_t=String::from("pgmq_latest_queue_name"))]
    pub pgmq_latest_queue_name: String,

    #[arg(long, default_value_t=String::from("pgmq_backfill_queue_name"))]
    pub pgmq_backfill_queue_name: String,

    #[arg(long, default_value_t=String::from("pgmq_notifications_queue_name"))]
    pub pgmq_notifications_queue_name: String,

    /// URL for pgmq.
    #[arg(long, default_value_t=String::from("127.0.0.1:5433"))]
    pub pgmq_url: String,

    #[clap(flatten)]
    pub shared: ShareableArgsValues,

    #[arg(long)]
    pub gemini_api_key: String,

    #[arg(long)]
    pub hardcover_api_key: String,

    #[arg(long, default_value_t = 365)]
    pub cookie_and_session_duration_days: i64,

    // DEFAULT LOGIC:
    // 1. If user provides --audiobook-cache-ttl, use that.
    // 2. Else if "environment" == "prod", use "3600" (1 hour).
    // 3. Else, use 5 seconds
    #[arg(long, default_value_t = 5)]
    #[arg(default_value_if("environment", "prod", "3600"))]
    pub audiobook_cache_ttl: u64,

    #[arg(long, default_value_t = 5)]
    #[arg(default_value_if("environment", "prod", "5000"))]
    pub audiobook_cache_max_capacity: u64,

    #[arg(long, default_value_t = 1)]
    #[arg(default_value_if("environment", "prod", "360"))]
    pub meta_cache_ttl: u64,

    #[arg(long, default_value_t = 1)]
    #[arg(default_value_if("environment", "prod", "360"))]
    pub meta_cache_max_capacity: u64,
}
