use serde_json::Value;
use shared::private_args::Args;
use shared::utils::gemini::{GeminiContentEmbedderLike, GeminiContentGeneratorLike};
use tracing::{debug, instrument};

use crate::scraping::prompts::{
    CREATE_DESCRIPTION_FOR_EMBEDDING, CREATE_VERY_SHORT_DESCRIPTION_PROMPT,
    PARSE_HTML_INSTRUCTIONS, get_audiobook_schema,
};
use crate::scraping::utils::html::extract_only_post_info;
use crate::scraping::utils::http::{build_robust_client, get_with_retries};

/// # Errors
/// - If `get_with_retries` fails.
///
/// # Panics
/// - If the data extracted can't be converted to string.
#[instrument(skip_all)]
pub async fn extract_data_from_html<T: GeminiContentGeneratorLike>(
    gemini: &T,
    url: &str,
    args: &Args,
) -> Result<Value, Box<dyn std::error::Error>> {
    let client = build_robust_client();

    let res = get_with_retries(
        &client,
        url,
        &args.shared.audiobookbay_domain,
        &args.shared.audiobookbay_extensions,
    )
    .await?
    .text()
    .await
    .unwrap();

    debug!("Res body when getting the page: {}", &res);
    let post_information = extract_only_post_info(&res).ok_or("Couldn't get the post info")?;
    let fomatted_prompt = PARSE_HTML_INSTRUCTIONS
        .to_string()
        .replace("{html}", &post_information);

    gemini
        .generate_structured_content(&fomatted_prompt, get_audiobook_schema())
        .await
        .map_err(|e| e.to_string().into())
}

/// # Errors
///   - If the content generation fails.
#[instrument(skip_all)]
pub async fn create_short_description<T: GeminiContentGeneratorLike>(
    gemini: &T,
    maybe_description: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(description) = maybe_description {
        let formatted_prompt = CREATE_VERY_SHORT_DESCRIPTION_PROMPT
            .to_string()
            .replace("{description}", description);
        return gemini
            .generate_content(&formatted_prompt)
            .await
            .map_err(|e| e.to_string().into());
    }
    debug!("Description is not available. Skipping.");
    Ok("Description not available".to_string())
}

/// # Errors
///   - If the content generation fails.
#[instrument(skip_all)]
pub async fn create_embeddable_description<T: GeminiContentGeneratorLike>(
    gemini: &T,
    maybe_description: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(description) = maybe_description {
        let formatted_prompt = CREATE_DESCRIPTION_FOR_EMBEDDING
            .to_string()
            .replace("{description}", description);
        return gemini
            .generate_content(&formatted_prompt)
            .await
            .map_err(|e| e.to_string().into());
    }
    debug!("Description is not available. Skipping.");
    Ok("Description not available".to_string())
}

/// # Errors
///   - If the embedding creation fails either due to hitting the Google API fails, or because
///     we the normalization fails.
#[instrument(skip_all)]
pub async fn create_embeddings<T: GeminiContentEmbedderLike>(
    gemini: &T,
    description: &str,
    args: &Args,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let formatted_prompt = CREATE_DESCRIPTION_FOR_EMBEDDING
        .to_string()
        .replace("{description}", description);
    return gemini
        .embed_content(
            &formatted_prompt,
            &gemini_rust::TaskType::RetrievalDocument,
            args.shared.gemini_embeddings_size,
        )
        .await
        .map_err(|e| e.to_string().into());
}
