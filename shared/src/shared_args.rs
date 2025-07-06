use clap;

#[derive(Clone, Debug, clap::ValueEnum, PartialEq)]
pub enum Environment {
    DEV,
    PROD,
}
#[derive(clap::Args, Clone, Debug)]
pub struct ShareableArgsValues {
    #[clap(value_enum)]
    #[arg(long, default_value_t = Environment::DEV)]
    pub environment: Environment,

    #[arg(long, default_value_t = 24)]
    pub guest_user_audiobooks_per_homepage: u16,

    #[arg(long, default_value_t = 8)]
    pub user_audiobooks_per_homepage_section: u16,
}
