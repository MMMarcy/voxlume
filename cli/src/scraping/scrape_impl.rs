use gemini_rust::Gemini;
use pgmq::PGMQueue;
use shared::db_ops::parade::search_ops::does_audiobook_exists;
use shared::private_args::Args;
use shared::utils::gemini::GeminiContentGeneratorLike;
use sqlx::PgPool;
use tracing::{debug, info, instrument};
use url::Url;

use crate::queue_messages::IngestedAudiobookMessage;
use crate::scraping::prompts::{PARSE_HTML_INSTRUCTIONS, get_submission_list_schema};
use crate::scraping::queue_items::QueueTask;
use crate::scraping::utils::db::save_audiobook_transaction;
use crate::scraping::utils::gemini_extraction::{
    create_embeddable_description, create_embeddings, create_short_description,
    extract_data_from_html,
};
use crate::scraping::utils::hardcover_ops::resolve_hardcover_metadata;
use crate::scraping::utils::html::extract_only_new_submissions_table;
use crate::scraping::utils::http::{build_robust_client, get_with_retries};

#[instrument(skip_all)]
pub async fn handle_submission_page(
    args: &Args,
    pgpool: &PgPool,
    queue: &PGMQueue,
    queue_name: &str,
    url: String,
    has_made_http_request: &mut bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let g = Gemini::with_model(
        &args.gemini_api_key,
        args.shared.gemini_extract_html_model_name.clone(),
    )
    .unwrap();
    let client = build_robust_client();
    let res = get_with_retries(
        &client,
        &url,
        &args.shared.audiobookbay_domain,
        &args.shared.audiobookbay_extensions,
    )
    .await?
    .text()
    .await
    .unwrap();
    debug!("Res body: {res}");
    let submission_table =
        extract_only_new_submissions_table(&res).ok_or("Couldn't get the new items table")?;
    let fomatted_prompt = PARSE_HTML_INSTRUCTIONS
        .to_string()
        .replace("{html}", &submission_table);

    let base_str = format!(
        "https://{}.{}/",
        args.shared.audiobookbay_domain,
        args.shared
            .audiobookbay_extensions
            .first()
            .expect("At least one extension must be provided")
    );

    let res = g
        .generate_structured_content(&fomatted_prompt, get_submission_list_schema(&base_str))
        .await?;
    let base_url = Url::parse(&base_str)?;

    if let Some(submissions) = res["submissions"].as_array() {
        let rev_subs: Vec<_> = submissions.iter().rev().collect();
        for submission in rev_subs {
            let url = base_url
                .join(submission["url"].as_str().unwrap_or(""))?
                .to_string();
            let audiobook_exists = does_audiobook_exists(pgpool, &url)
                .await
                .map_err(|e| e.to_string())?;
            if audiobook_exists {
                info!("Skipping enqueueing audiobook as it already exists");
                *has_made_http_request = true;
                continue;
            }
            let submission_date = submission["submission_date"]
                .as_str()
                .unwrap_or("")
                .to_string();
            info!("Enqueueing audiobook page {}", &url);
            let queue_task = QueueTask::ParseAudiobookPage {
                url,
                submission_date,
            };
            _ = queue.send(queue_name, &queue_task).await?;
        }
    }
    *has_made_http_request = true;

    Ok(())
}

#[instrument(skip_all)]
pub async fn handle_audiobook_page(
    args: &Args,
    pool: &PgPool,
    queue: &PGMQueue,
    url: String,
    has_made_http_request: &mut bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let notification_queue_name = &args.pgmq_notifications_queue_name;
    let audiobook_exists = does_audiobook_exists(pool, &url)
        .await
        .map_err(|e| e.to_string())?;
    if audiobook_exists {
        info!("Audiobook at url ({})exists, skipping...", &url);
        *has_made_http_request = false;
        return Ok(());
    }
    let g = Gemini::with_model(
        &args.gemini_api_key,
        args.shared.gemini_extract_html_model_name.clone(),
    )
    .unwrap();
    let g_emb = Gemini::with_model(
        &args.gemini_api_key,
        args.shared.gemini_embedding_model_name.clone(),
    )
    .unwrap();

    let extracted_values = extract_data_from_html(&g, &url, args).await?;
    info!(
        "Extracted values length {}",
        extracted_values.to_string().len()
    );
    debug!("Raw values: {:?}", extracted_values);

    let maybe_description = extracted_values["description"].as_str();
    let short_description = create_short_description(&g, maybe_description).await?;
    info!("Short description: {}", &short_description);

    let embeddable_description = create_embeddable_description(&g, maybe_description).await?;
    debug!("Embeddable description: {}", &embeddable_description);

    let embeddings = create_embeddings(&g_emb, &embeddable_description, args).await?;

    let hardcover_book = resolve_hardcover_metadata(args, &extracted_values).await;

    let audiobook_id = save_audiobook_transaction(
        pool,
        &url,
        &extracted_values,
        &short_description,
        &embeddable_description,
        embeddings,
        hardcover_book,
    )
    .await?;

    info!("Successfully inserted audiobook: {}", &url);

    let message = IngestedAudiobookMessage::new(audiobook_id);
    queue.send(notification_queue_name, &message).await?;
    info!(
        "Successfully propagated audiobook_id to queue {}",
        notification_queue_name
    );

    Ok(())
}
