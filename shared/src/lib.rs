use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;

use crate::private_args::Args;
use entities_lib::Environment;

pub mod auth_user;
pub mod db_ops;
pub mod db_trait;
pub mod password_handler;
pub mod private_args;
pub mod sql_user;
pub mod state;
pub mod utils;

pub fn configure_tracing(args: &Args) {
    match args.shared.environment {
        // TODO: In dev it is a bit too verbose. Especially the timestamp field. Find a way to
        // format that.
        Environment::DEV => tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .pretty()
            .with_line_number(true)
            .with_target(true)
            .with_span_events(FmtSpan::CLOSE) // Or FmtSpan::ENTER, FmtSpan::CLOSE etc.
            .init(),

        // TODO: Add a sink to disk for logs.
        Environment::PROD => tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_span_events(FmtSpan::CLOSE) // Or FmtSpan::ENTER, FmtSpan::CLOSE etc.
            .json()
            .with_current_span(false)
            .init(),
    }
}
