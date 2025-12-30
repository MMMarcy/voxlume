use chrono::{DateTime, Utc};
use entities_lib::{
    AudioBook, AudiobookWithData, Author, Category, GetAudioBookRequestType, Keyword, Reader,
    Series,
};
use moka::future::Cache;
use sqlx::postgres::PgArguments;
use sqlx::{Arguments, FromRow, PgPool};
use tracing::instrument;

use crate::db_ops::AppError;

/// .
///
/// # Errors
///
/// This function will return an error if.
#[instrument(skip_all)]
pub async fn get_audiobooks_cached(
    db_pool: &PgPool,
    cache: &Cache<GetAudioBookRequestType, Result<Vec<AudiobookWithData>, AppError>>,
    request_type: GetAudioBookRequestType,
    limit: u32,
) -> Result<Vec<AudiobookWithData>, AppError> {
    let cache_key = request_type.clone();

    match request_type {
        GetAudioBookRequestType::MostRecent(page) => {
            cache
                .get_with(cache_key, async {
                    get_most_recent_audiobooks_with_data(db_pool, limit, page).await
                })
                .await
        }
        GetAudioBookRequestType::ByAuthor(author, page) => {
            cache
                .get_with(cache_key, async {
                    get_audiobooks_with_data_by_author(db_pool, author, limit, page).await
                })
                .await
        }
        GetAudioBookRequestType::ByReader(reader, page) => {
            cache
                .get_with(cache_key, async {
                    get_audiobooks_with_data_by_reader(db_pool, reader, limit, page).await
                })
                .await
        }
        GetAudioBookRequestType::ByCategory(category, page) => {
            cache
                .get_with(cache_key, async {
                    get_audiobooks_with_data_by_category(db_pool, category, limit, page).await
                })
                .await
        }
        GetAudioBookRequestType::ByKeyword(keyword, page) => {
            cache
                .get_with(cache_key, async {
                    get_audiobooks_with_data_by_keyword(db_pool, keyword, limit, page).await
                })
                .await
        }
        GetAudioBookRequestType::BySeries(series, page) => {
            cache
                .get_with(cache_key, async {
                    get_audiobooks_with_data_by_series(db_pool, series, limit, page).await
                })
                .await
        }
        GetAudioBookRequestType::ById(id) => {
            cache
                .get_with(cache_key, async { get_audiobook_by_id(db_pool, id).await })
                .await
        }
        GetAudioBookRequestType::ByIdList(ids_list) => {
            cache
                .get_with(cache_key, async {
                    get_audiobooks_by_ids(
                        db_pool,
                        ids_list
                            .iter()
                            .map(std::string::ToString::to_string)
                            .collect(),
                    )
                    .await
                })
                .await
        }
        GetAudioBookRequestType::AllExcept(_categories, _keywords) => todo!(),
    }
}

const BASE_AUDIOBOOK_QUERY: &str = "
SELECT
    ab.id, ab.title, ab.bitrate, ab.cover_url, ab.description, ab.very_short_description,
    ab.description_for_embeddings, ab.file_size, ab.format, ab.language, ab.path,
    ab.timestamp_ingested, ab.unabridged,
    COALESCE(array_agg(DISTINCT aut.id) FILTER (WHERE aut.name IS NOT NULL), '{}') AS authors_ids,
    COALESCE(array_agg(DISTINCT aut.name) FILTER (WHERE aut.name IS NOT NULL), '{}') AS authors,
    COALESCE(array_agg(DISTINCT cat.id) FILTER (WHERE cat.name IS NOT NULL), '{}') AS categories_ids,
    COALESCE(array_agg(DISTINCT cat.name) FILTER (WHERE cat.name IS NOT NULL), '{}') AS categories,
    COALESCE(array_agg(DISTINCT key.id) FILTER (WHERE key.name IS NOT NULL), '{}') AS keywords_ids,
    COALESCE(array_agg(DISTINCT key.name) FILTER (WHERE key.name IS NOT NULL), '{}') AS keywords,
    COALESCE(array_agg(DISTINCT rdr.id) FILTER (WHERE rdr.name IS NOT NULL), '{}') AS readers_ids,
    COALESCE(array_agg(DISTINCT rdr.name) FILTER (WHERE rdr.name IS NOT NULL), '{}') AS readers,
    ser.id AS series_id,
    ser.title as series_title
FROM audiobook ab
LEFT JOIN audiobook_author aba ON ab.id = aba.audiobook_id
LEFT JOIN author aut ON aba.author_id = aut.id
LEFT JOIN audiobook_category abc ON ab.id = abc.audiobook_id
LEFT JOIN category cat ON abc.category_id = cat.id
LEFT JOIN audiobook_keyword abk ON ab.id = abk.audiobook_id
LEFT JOIN keyword key ON abk.keyword_id = key.id
LEFT JOIN audiobook_reader abr ON ab.id = abr.audiobook_id
LEFT JOIN reader rdr ON abr.reader_id = rdr.id
LEFT JOIN series ser ON ab.series_id = ser.id
";

const GROUP_BY_AUDIOBOOK: &str = "GROUP BY ab.id, ser.title, ser.id";

#[derive(FromRow)]
struct FullAudiobookRow {
    id: i64,
    title: String,
    bitrate: Option<i32>,
    cover_url: String,
    description: String,
    very_short_description: String,
    description_for_embeddings: String,
    file_size: Option<i64>,
    format: String,
    language: String,
    path: String,
    timestamp_ingested: DateTime<Utc>,
    unabridged: Option<bool>,
    authors_ids: Vec<i64>,
    authors: Vec<String>,
    categories_ids: Vec<i64>,
    categories: Vec<String>,
    keywords_ids: Vec<i64>,
    keywords: Vec<String>,
    readers_ids: Vec<i64>,
    readers: Vec<String>,
    series_id: Option<i64>,
    series_title: Option<String>,
}

fn map_row_to_audiobook_with_data(row: FullAudiobookRow) -> AudiobookWithData {
    let audiobook = AudioBook {
        id: row.id,
        title: row.title,
        bitrate: row.bitrate.map(|b| b.to_string()),
        categories: row.categories.clone(),
        cover_url: Some(row.cover_url),
        description: row.description,
        very_short_description: row.very_short_description,
        description_for_embeddings: row.description_for_embeddings,
        file_size: row.file_size.map(|fs| fs.to_string()),
        format: Some(row.format),
        keywords: row.keywords.clone(),
        language: row.language,
        path: row.path,
        last_upload: row.timestamp_ingested.timestamp(),
        unabriged: row.unabridged.unwrap_or(false),
        series_volume: None,
    };

    let authors = row
        .authors_ids
        .into_iter()
        .zip(row.authors)
        .map(|values: (i64, String)| Author {
            id: values.0,
            name: values.1,
        })
        .collect();
    let categories = row
        .categories_ids
        .into_iter()
        .zip(row.categories)
        .map(|values: (i64, String)| Category {
            id: values.0,
            value: values.1,
        })
        .collect();
    let keywords = row
        .keywords_ids
        .into_iter()
        .zip(row.keywords)
        .map(|values: (i64, String)| Keyword {
            id: values.0,
            value: values.1,
        })
        .collect();
    let readers = row
        .readers_ids
        .into_iter()
        .zip(row.readers)
        .map(|values: (i64, String)| Reader {
            id: values.0,
            name: values.1,
        })
        .collect();
    let maybe_series_id = row.series_id;
    let maybe_series_title = row.series_title;
    let series = match (maybe_series_id, maybe_series_title) {
        (Some(id), Some(title)) => Some(Series { id, title }),
        _ => None,
    };

    (audiobook, authors, categories, keywords, readers, series)
}

async fn execute_query(
    db_pool: &PgPool,
    query: &str,
    params: PgArguments,
) -> Result<Vec<AudiobookWithData>, AppError> {
    let query_builder = sqlx::query_as_with::<_, FullAudiobookRow, _>(query, params);
    let rows = query_builder
        .fetch_all(db_pool)
        .await
        .map_err(|e| AppError::DeserializationError(format!("Database query failed: {e}")))?;

    Ok(rows
        .into_iter()
        .map(map_row_to_audiobook_with_data)
        .collect())
}

#[instrument(skip_all)]
async fn get_audiobooks_by_ids(
    db_pool: &PgPool,
    ids: Vec<String>,
) -> Result<Vec<AudiobookWithData>, AppError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let ids_i64: Result<Vec<i64>, _> = ids.iter().map(|id| id.parse()).collect();
    let ids_i64 =
        ids_i64.map_err(|e| AppError::DeserializationError(format!("Invalid ID format: {e}")))?;

    let query_str = format!("{BASE_AUDIOBOOK_QUERY} WHERE ab.id = ANY($1) {GROUP_BY_AUDIOBOOK}");

    let mut args = PgArguments::default();
    let _ = args.add(&ids_i64);

    let mut audiobooks = execute_query(db_pool, &query_str, args).await?;

    // The database query does not respect the order of the input IDs (e.g. from search relevance).
    // We must manually resort the results to match the 'ids' vector order.
    let position_map: std::collections::HashMap<i64, usize> = ids_i64
        .iter()
        .enumerate()
        .map(|(idx, id)| (*id, idx))
        .collect();

    audiobooks.sort_by_key(|(ab, ..)| position_map.get(&ab.id).copied().unwrap_or(usize::MAX));

    Ok(audiobooks)
}

#[instrument(skip_all)]
async fn get_audiobook_by_id(
    db_pool: &PgPool,
    id: String,
) -> Result<Vec<AudiobookWithData>, AppError> {
    get_audiobooks_by_ids(db_pool, vec![id]).await
}

async fn get_paginated_audiobooks_by_relation_using_ids(
    db_pool: &PgPool,
    relation_table: &str,
    relation_column: &str,
    relation_value: i64,
    join_table: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<AudiobookWithData>, AppError> {
    let query_str = format!(
        "WITH filtered_ab AS (
            SELECT ab.id FROM audiobook ab
            JOIN {join_table} j ON ab.id = j.audiobook_id
            JOIN {relation_table} r ON j.{relation_table}_id = r.id
            WHERE r.{relation_column} = $1
            ORDER BY ab.timestamp_ingested DESC
            LIMIT $2 OFFSET $3
        )
        {BASE_AUDIOBOOK_QUERY}
        JOIN filtered_ab ON ab.id = filtered_ab.id
        {GROUP_BY_AUDIOBOOK}
        ORDER BY ab.timestamp_ingested DESC"
    );

    let mut args = PgArguments::default();
    let _ = args.add(relation_value);
    let _ = args.add(limit);
    let _ = args.add(offset);
    execute_query(db_pool, &query_str, args).await
}

#[instrument(skip_all)]
async fn get_audiobooks_with_data_by_category(
    db_pool: &PgPool,
    category: Category,
    limit: u32,
    page: u32,
) -> Result<Vec<AudiobookWithData>, AppError> {
    let limit = i64::from(limit);
    let offset = i64::from(page.saturating_sub(1)) * limit;
    get_paginated_audiobooks_by_relation_using_ids(
        db_pool,
        "category",
        "id",
        category.id,
        "audiobook_category",
        limit,
        offset,
    )
    .await
}

#[instrument(skip_all)]
async fn get_audiobooks_with_data_by_keyword(
    db_pool: &PgPool,
    keyword: Keyword,
    limit: u32,
    page: u32,
) -> Result<Vec<AudiobookWithData>, AppError> {
    let limit = i64::from(limit);
    let offset = i64::from(page.saturating_sub(1)) * limit;
    get_paginated_audiobooks_by_relation_using_ids(
        db_pool,
        "keyword",
        "id",
        keyword.id,
        "audiobook_keyword",
        limit,
        offset,
    )
    .await
}

#[instrument(skip_all)]
async fn get_audiobooks_with_data_by_series(
    db_pool: &PgPool,
    series: Series,
    limit: u32,
    page: u32,
) -> Result<Vec<AudiobookWithData>, AppError> {
    let limit = i64::from(limit);
    let offset = i64::from(page.saturating_sub(1)) * limit;
    let query_str = format!(
        "WITH filtered_ab AS (
            SELECT ab.id FROM audiobook ab
            JOIN series s ON ab.series_id = s.id
            WHERE s.id = $1
            ORDER BY ab.timestamp_ingested DESC
            LIMIT $2 OFFSET $3
        )
        {BASE_AUDIOBOOK_QUERY}
        JOIN filtered_ab ON ab.id = filtered_ab.id
        {GROUP_BY_AUDIOBOOK}
        ORDER BY ab.timestamp_ingested DESC"
    );

    let mut args = PgArguments::default();
    let _ = args.add(series.id);
    let _ = args.add(limit);
    let _ = args.add(offset);
    execute_query(db_pool, &query_str, args).await
}

#[instrument(skip_all)]
async fn get_audiobooks_with_data_by_reader(
    db_pool: &PgPool,
    reader: Reader,
    limit: u32,
    page: u32,
) -> Result<Vec<AudiobookWithData>, AppError> {
    let limit = i64::from(limit);
    let offset = i64::from(page.saturating_sub(1)) * limit;
    get_paginated_audiobooks_by_relation_using_ids(
        db_pool,
        "reader",
        "id",
        reader.id,
        "audiobook_reader",
        limit,
        offset,
    )
    .await
}

#[instrument(skip_all)]
async fn get_audiobooks_with_data_by_author(
    db_pool: &PgPool,
    author: Author,
    limit: u32,
    page: u32,
) -> Result<Vec<AudiobookWithData>, AppError> {
    let limit = i64::from(limit);
    let offset = i64::from(page.saturating_sub(1)) * limit;
    get_paginated_audiobooks_by_relation_using_ids(
        db_pool,
        "author",
        "id",
        author.id,
        "audiobook_author",
        limit,
        offset,
    )
    .await
}

#[instrument(skip_all)]
async fn get_most_recent_audiobooks_with_data(
    db_pool: &PgPool,
    limit: u32,
    page: u32,
) -> Result<Vec<AudiobookWithData>, AppError> {
    let limit = i64::from(limit);
    let offset = i64::from(page.saturating_sub(1)) * limit;
    let query_str = format!(
        "{BASE_AUDIOBOOK_QUERY}
        {GROUP_BY_AUDIOBOOK}
        ORDER BY ab.timestamp_ingested DESC
        LIMIT $1 OFFSET $2"
    );

    let mut args = PgArguments::default();
    let _ = args.add(limit);
    let _ = args.add(offset);
    execute_query(db_pool, &query_str, args).await
}
