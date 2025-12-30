#![recursion_limit = "256"]

use clap::Parser;
use shared::private_args::Args;
use tracing::{debug, info};

mod handlers;
mod setup;

#[allow(clippy::let_unit_value)]
#[tokio::main]
async fn main() {
    let args = Args::parse();
    let _guard = shared::configure_tracing(&args);

    debug!("Args: {:?}", &args);
    let (router, addr) = setup::init_app(&args).await;

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("listening on http://{}", &addr);
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
}
