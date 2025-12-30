use pgmq::{PGMQueue, PgmqError};
use tracing::{error, info};

use crate::private_args::Args;

/// Connects to PGMQ and creates the queue (if it doesn't exists).
///
/// # Errors
/// If the queue creation fails.
///
/// # Panics
/// If it can't connect to PGMQ.
pub async fn get_pgmq_queue(args: &Args, queue_name: &str) -> Result<PGMQueue, PgmqError> {
    let connection_string = format!(
        "postgres://{user}:{password}@{url}/{db}",
        user = args.pgmq_username.clone(),
        password = args.pgmq_password.clone(),
        url = args.pgmq_url.clone(),
        db = args.pgmq_database.clone()
    );
    let queue: PGMQueue = PGMQueue::new(connection_string.clone())
        .await
        .expect("Failed to connect to postgres");

    match queue.create(queue_name).await {
        Ok(()) => {
            info!("PgmqQueue {} was successfully created", queue_name);
            Ok(queue)
        }
        Err(e) => {
            error!("{}", e);
            Err(e)
        }
    }
}
