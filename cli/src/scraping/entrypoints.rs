use std::time::Duration;

use pgmq::PGMQueue;
use shared::db_ops::parade::get_postgres_connection;
use shared::db_ops::pgmq::get_pgmq_queue;
use shared::private_args::Args;
use sqlx::PgPool;
use tokio::time::sleep;
use tracing::{debug, error, info};
use url::Url;

use crate::scraping::queue_items::QueueTask;
use crate::scraping::scrape_impl::{handle_audiobook_page, handle_submission_page};

async fn dequeue_and_do_work(
    pgpool: &PgPool,
    queue: &PGMQueue,
    queue_name: &str,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let mut has_made_http_request: bool = true;
        let maybe_queue_item = queue.pop::<QueueTask>(queue_name).await?;
        if maybe_queue_item.is_none() {
            return Ok(());
        }
        debug!("Found element in queue.");
        match maybe_queue_item.unwrap().message {
            QueueTask::ParseSubmissionPage(url) => {
                info!("Parsing submission page {}", &url);
                handle_submission_page(
                    args,
                    pgpool,
                    queue,
                    queue_name,
                    url,
                    &mut has_made_http_request,
                )
                .await?;
                debug!(
                    "After parsing submission page `has_made_http_request` is {}",
                    &has_made_http_request
                );
            }
            QueueTask::ParseAudiobookPage {
                url,
                submission_date: _,
            } => {
                if !&url
                    .to_lowercase()
                    .contains(&args.shared.audiobookbay_domain)
                {
                    error!(
                        "URL {} doesn't contain the base path {}",
                        &url, &args.shared.audiobookbay_domain
                    );
                    continue;
                }
                info!("Parsing audiobook page {}", &url);
                match handle_audiobook_page(args, pgpool, queue, url, &mut has_made_http_request)
                    .await
                {
                    Ok(e) => debug!("{e:?}"),
                    Err(e) => error!("{e:?}"),
                }
                debug!(
                    "After parsing audiobookbay page `has_made_http_request` is {}",
                    &has_made_http_request
                );
            }
        }
        if has_made_http_request {
            sleep(Duration::from_secs(30)).await;
        }
    }
}

/// Function that continues scraping the latest page of audiobooks.
///
/// # Errors
///   - If it is not possible to create a queue with the given configuration.
///   - If adding an item to the queue fails.
///   - If the `dequeue_and_do_work` fails.
/// # Panics
///   - If the list of audiobookbay extensions is empty.
pub async fn handle_scrape_impl(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let queue = get_pgmq_queue(&args, &args.pgmq_latest_queue_name).await?;
    let queue_name = args.pgmq_latest_queue_name.clone();

    let base_str = format!(
        "https://{}.{}/",
        args.shared.audiobookbay_domain,
        args.shared
            .audiobookbay_extensions
            .first()
            .expect("At least one extension must be provided")
    );
    let base = Url::parse(&base_str)?;
    let url: String = base.join("member/index?pid=1")?.to_string();
    let pgpool = get_postgres_connection(&args).await;

    let task = QueueTask::ParseSubmissionPage(url);
    loop {
        _ = queue.send(&queue_name, &task).await?;
        dequeue_and_do_work(&pgpool, &queue, &queue_name, &args).await?;
        info!("Queue is empty, so sleeping for 1800 seconds");
        sleep(Duration::from_secs(1800)).await;
    }
}
/// Function that backfills the data from audiobookbay.
///
/// # Errors
///   - If it is not possible to create a queue with the given configuration.
///   - If adding an item to the queue fails.
///   - If the `dequeue_and_do_work` fails.
/// # Panics
///   - If the list of audiobookbay extensions is empty.
pub async fn handle_backfill_impl(
    args: Args,
    page_start: u16,
    page_end: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let queue = get_pgmq_queue(&args, &args.pgmq_backfill_queue_name).await?;
    let queue_name = args.pgmq_backfill_queue_name.clone();
    let base_str = format!(
        "https://{}.{}/",
        args.shared.audiobookbay_domain,
        args.shared
            .audiobookbay_extensions
            .first()
            .expect("At least one extension must be provided")
    );
    let base_url = Url::parse(&base_str)?;
    let pgpool = get_postgres_connection(&args).await;
    for page in (page_start..page_end).rev() {
        let url: String = base_url
            .join(&format!("member/index?pid={page}"))?
            .to_string();
        debug!(
            "Adding submission page (url = {}) to queue {}",
            &url, &queue_name
        );

        let task = QueueTask::ParseSubmissionPage(url);
        _ = queue.send(&queue_name, &task).await?;

        dequeue_and_do_work(&pgpool, &queue, &queue_name, &args).await?;
    }
    Ok(())
}
