use clap::{Parser, ValueEnum};

#[derive(Clone, Debug, ValueEnum)]
pub enum Environment {
    DEV,
    PROD,
}

#[derive(Debug, Parser, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Username for neo4j.
    #[arg(long, default_value_t=String::from("neo4j"))]
    pub neo4j_username: String,

    /// Username for neo4j.
    #[arg(long, default_value_t=String::from("password"))]
    pub neo4j_password: String,

    /// URL for neo4j.
    #[arg(long, default_value_t=String::from("127.0.0.1:7687"))]
    pub neo4j_url: String,

    /// Username for postgres.
    #[arg(long, default_value_t=String::from("postgres"))]
    pub postgres_username: String,

    /// Username for postgres.
    #[arg(long, default_value_t=String::from("password"))]
    pub postgres_password: String,

    /// URL for postgres.
    #[arg(long, default_value_t=String::from("127.0.0.1:5432"))]
    pub postgres_url: String,

    #[clap(value_enum)]
    #[arg(long, default_value_t=Environment::DEV)]
    pub environment: Environment,
}
