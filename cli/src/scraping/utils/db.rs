use serde_json::Value;
use shared::db_ops::parade::save_ops::insert_audiobook_data;
use sqlx::PgPool;
use tracing::info;

use crate::scraping::hardcover::HardcoverBook;

/// # Errors
///   - If the storing of the audiobook on the db fails.
///   - If the storing of hardcover information on the db fails.
///
/// # Panics
///   - If the serialization of the hardcover data fails.
pub async fn save_audiobook_transaction(
    pool: &PgPool,
    url: &str,
    extracted_values: &Value,
    short_description: &str,
    embeddable_description: &str,
    embeddings: Vec<f32>,
    hardcover_book: Option<HardcoverBook>,
) -> Result<i64, Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;
    let audiobook_id = insert_audiobook_data(
        &mut tx,
        url,
        extracted_values,
        short_description,
        embeddable_description,
        embeddings,
    )
    .await?;

    if let Some(book) = hardcover_book {
        sqlx::query!(
            "INSERT INTO hardcover_audiobook_metadata (audiobook_id, metadata) VALUES ($1, $2)",
            audiobook_id,
            serde_json::to_value(&book).unwrap()
        )
        .execute(&mut *tx)
        .await?;
        info!("Inserted hardcover metadata for audiobook {}", audiobook_id);
    }

    tx.commit().await?;
    Ok(audiobook_id)
}
