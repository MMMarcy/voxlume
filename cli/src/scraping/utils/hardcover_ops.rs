use serde_json::Value;
use shared::private_args::Args;
use tracing::error;

use crate::scraping::hardcover::{search_book, HardcoverBook};
use crate::scraping::utils::http::build_robust_client;

pub async fn resolve_hardcover_metadata(
    args: &Args,
    extracted_values: &Value,
) -> Option<HardcoverBook> {
    let title = extracted_values["title"].as_str().unwrap_or("").to_string();
    let authors_val = extracted_values["authors"].as_array();
    let author = if let Some(authors) = authors_val {
        if let Some(first_author) = authors.first() {
            first_author.as_str().unwrap_or("").to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    if !title.is_empty() && !author.is_empty() {
        match search_book(
            &build_robust_client(),
            &args.hardcover_api_key,
            &title,
            &author,
        )
        .await
        {
            Ok(book) => book,
            Err(e) => {
                error!("Failed to search hardcover: {}", e);
                None
            }
        }
    } else {
        None
    }
}
