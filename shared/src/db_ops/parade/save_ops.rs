use chrono::{Local, NaiveDateTime, TimeZone, Utc};
use serde_json::Value;
use sqlx::Transaction;
use std::error::Error;
use tracing::instrument;

fn parse_bitrate(s: &str) -> Option<i32> {
    s.trim()
        .to_lowercase()
        .strip_suffix("kbps")?
        .trim()
        .parse::<i32>()
        .ok()
}

#[allow(clippy::cast_possible_truncation)]
fn parse_filesize(s: &str) -> Option<i64> {
    let s = s.trim().to_lowercase();
    if let Some(s_gb) = s.strip_suffix("gb") {
        s_gb.trim()
            .parse::<f64>()
            .ok()
            .map(|gb| (gb * 1_000_000_000.0) as i64)
    } else if let Some(s_mb) = s.strip_suffix("mb") {
        s_mb.trim()
            .parse::<f64>()
            .ok()
            .map(|mb| (mb * 1_000_000.0) as i64)
    } else {
        s.trim().parse::<i64>().ok()
    }
}

/// Insert the audiobook data into the DB.
///
/// # Errors
/// If the insert doesn't work for some reason.
#[instrument(skip_all)]
pub async fn insert_audiobook_data(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    url: &str,
    extracted_values: &Value,
    short_description: &str,
    embeddable_description: &str,
    embeddings: Vec<f32>,
) -> Result<i64, Box<dyn Error>> {
    let series_id: Option<i64> = if let Some(series_title) = extracted_values["series"]
        .as_str()
        .filter(|s| !s.is_empty())
    {
        Some(
                sqlx::query_scalar(
                "INSERT INTO series (title) VALUES ($1) ON CONFLICT (title) DO UPDATE SET title = EXCLUDED.title RETURNING id",
            ).bind(series_title)
                .fetch_one(&mut **tx)
                .await?,
            )
    } else {
        None
    };
    let parsed_date = extracted_values["upload_date"].as_str().unwrap_or("");
    let naive_date =
        NaiveDateTime::parse_from_str(parsed_date, "%Y-%m-%d").unwrap_or(Local::now().naive_utc());
    let timestamp_created = Utc.from_utc_datetime(&naive_date);

    let audiobook_id: i64 = sqlx::query_scalar(
        r"
        INSERT INTO audiobook (
            title, language, cover_url, format, unabridged, description, bitrate, file_size, series_id,
            path, timestamp_created, timestamp_ingested, very_short_description,
            description_for_embeddings, optimized_description_embedding
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), $12, $13, $14)
        RETURNING id
        ")
        .bind(extracted_values["title"].as_str())
        .bind(extracted_values["language"].as_str())
        .bind(extracted_values["cover_url"].as_str())
        .bind(extracted_values["format"].as_str())
        .bind(extracted_values["unabridged"].as_bool())
        .bind(extracted_values["description"].as_str())
        .bind(extracted_values["bitrate"].as_str().and_then(parse_bitrate))
        .bind(extracted_values["file_size"].as_str().and_then(parse_filesize),)
        .bind(series_id)
        .bind(url)
        .bind(timestamp_created)
        .bind(short_description)
        .bind(embeddable_description)
        .bind(embeddings)
        .fetch_one(&mut **tx)
        .await?;

    for author in extracted_values["authors"].as_array().unwrap_or(&vec![]) {
        if let Some(author_name) = author.as_str() {
            let author_id: i64 = sqlx::query_scalar(
                "INSERT INTO author (name) VALUES ($1) ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name RETURNING id")
                .bind(author_name
            ).fetch_one(&mut **tx).await?;
            sqlx::query(
                "INSERT INTO audiobook_author (audiobook_id, author_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(audiobook_id)
                .bind(author_id)
            .execute(&mut **tx).await?;
        }
    }

    for reader in extracted_values["read_by"].as_array().unwrap_or(&vec![]) {
        if let Some(reader_name) = reader.as_str() {
            let reader_id: i64 = sqlx::query_scalar(
                "INSERT INTO reader (name) VALUES ($1) ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name RETURNING id")
                .bind(reader_name)
            .fetch_one(&mut **tx).await?;
            sqlx::query(
                "INSERT INTO audiobook_reader (audiobook_id, reader_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(audiobook_id)
                .bind(reader_id)
            .execute(&mut **tx).await?;
        }
    }

    for category in extracted_values["categories"].as_array().unwrap_or(&vec![]) {
        if let Some(category_name) = category.as_str() {
            let category_id: i64 = sqlx::query_scalar(
                "INSERT INTO category (name) VALUES ($1) ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name RETURNING id")
                .bind(category_name)
            .fetch_one(&mut **tx).await?;
            sqlx::query(
                "INSERT INTO audiobook_category (audiobook_id, category_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(audiobook_id)
                .bind(category_id)
            .execute(&mut **tx).await?;
        }
    }

    for keyword in extracted_values["keywords"].as_array().unwrap_or(&vec![]) {
        if let Some(keyword_name) = keyword.as_str() {
            let keyword_id: i64 = sqlx::query_scalar(
                "INSERT INTO keyword (name) VALUES ($1) ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name RETURNING id")
                .bind(keyword_name)
        .fetch_one(&mut **tx).await?;
            sqlx::query(
                "INSERT INTO audiobook_keyword (audiobook_id, keyword_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(audiobook_id)
                .bind(keyword_id)
            .execute(&mut **tx).await?;
        }
    }

    Ok(audiobook_id)
}
