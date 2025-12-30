use crate::db_ops::AppError;
use entities_lib::{Author, Category, Keyword, MetaRequest, MetaResponse, Reader, Series};
use moka::future::Cache;
use sqlx::{FromRow, PgPool};
use tracing::instrument;

#[derive(FromRow)]
struct IdNameRow {
    id: i64,
    name: String,
}

#[derive(FromRow)]
struct IdTitleRow {
    id: i64,
    title: String,
}

fn get_limit_offset(page: u32, limit: u32) -> (i64, i64) {
    let limit_val = i64::from(limit);
    let offset_val = i64::from(page.saturating_sub(1)) * limit_val;
    (limit_val, offset_val)
}

/// Returns the meta queries either from cache or by hitting the DB.
///
/// # Errors
/// - If the request is not present and so the query to hit the DB fails.
#[allow(clippy::too_many_lines)]
#[instrument(skip_all)]
pub async fn get_meta_cached(
    db_pool: &PgPool,
    cache: &Cache<MetaRequest, Result<MetaResponse, AppError>>,
    request: MetaRequest,
) -> Result<MetaResponse, AppError> {
    let key = request.clone();
    cache
        .get_with(key, async {
            match request {
                MetaRequest::CategoriesByPublishedAudiobooks(page, limit) => {
                    get_categories_by_published_audiobooks(db_pool, page, limit)
                        .await
                        .map(MetaResponse::Categories)
                }
                MetaRequest::KeywordsByPublishedAudiobooks(page, limit) => {
                    get_keywords_by_published_audiobooks(db_pool, page, limit)
                        .await
                        .map(MetaResponse::Keywords)
                }
                MetaRequest::AuthorsByPublishedAudiobooks(page, limit) => {
                    get_authors_by_published_audiobooks(db_pool, page, limit)
                        .await
                        .map(MetaResponse::Authors)
                }
                MetaRequest::ReaderByPublishedAudiobooks(page, limit) => {
                    get_readers_by_published_audiobooks(db_pool, page, limit)
                        .await
                        .map(MetaResponse::Readers)
                }
                MetaRequest::SeriesBySubscriber(page, limit) => {
                    // Maps to Series by Published Audiobooks count to complement the NSubscribers variant
                    get_series_by_published_audiobooks(db_pool, page, limit)
                        .await
                        .map(MetaResponse::Series)
                }
                MetaRequest::CategoriesByNSubscribers(page, limit) => {
                    get_categories_by_n_subscribers(db_pool, page, limit)
                        .await
                        .map(MetaResponse::Categories)
                }
                MetaRequest::KeywordsByNSubscribers(page, limit) => {
                    get_keywords_by_n_subscribers(db_pool, page, limit)
                        .await
                        .map(MetaResponse::Keywords)
                }
                MetaRequest::AuthorByNSubscribers(page, limit) => {
                    get_authors_by_n_subscribers(db_pool, page, limit)
                        .await
                        .map(MetaResponse::Authors)
                }
                MetaRequest::ReaderByNSubscribers(page, limit) => {
                    get_readers_by_n_subscribers(db_pool, page, limit)
                        .await
                        .map(MetaResponse::Readers)
                }
                MetaRequest::SeriesByNSubscribers(page, limit) => {
                    get_series_by_n_subscribers(db_pool, page, limit)
                        .await
                        .map(MetaResponse::Series)
                }
                MetaRequest::CategoriesAlphabetically(page, limit) => {
                    get_categories_alphabetically(db_pool, page, limit)
                        .await
                        .map(MetaResponse::Categories)
                }
                MetaRequest::KeywordsAlphabetically(page, limit) => {
                    get_keywords_alphabetically(db_pool, page, limit)
                        .await
                        .map(MetaResponse::Keywords)
                }
                MetaRequest::AuthorsAlphabetically(page, limit) => {
                    get_authors_alphabetically(db_pool, page, limit)
                        .await
                        .map(MetaResponse::Authors)
                }
                MetaRequest::ReadersAlphabetically(page, limit) => {
                    get_readers_alphabetically(db_pool, page, limit)
                        .await
                        .map(MetaResponse::Readers)
                }
                MetaRequest::SeriesAlphabetically(page, limit) => {
                    get_series_alphabetically(db_pool, page, limit)
                        .await
                        .map(MetaResponse::Series)
                }
                MetaRequest::CountAudiobooksForCategory(category) => {
                    count_audiobooks_for_category(db_pool, category.id)
                        .await
                        .map(MetaResponse::Count)
                }
                MetaRequest::CountAudiobooksForKeyword(keyword) => {
                    count_audiobooks_for_keyword(db_pool, keyword.id)
                        .await
                        .map(MetaResponse::Count)
                }
                MetaRequest::CountAudiobooksForAuthor(author) => {
                    count_audiobooks_for_author(db_pool, author.id)
                        .await
                        .map(MetaResponse::Count)
                }
                MetaRequest::CountAudiobooksForReader(reader) => {
                    count_audiobooks_for_reader(db_pool, reader.id)
                        .await
                        .map(MetaResponse::Count)
                }
                MetaRequest::CountAudiobooksInSeries(series) => {
                    count_audiobooks_in_series(db_pool, series.id)
                        .await
                        .map(MetaResponse::Count)
                }
                MetaRequest::CountAllAudiobooks => {
                    count_all_audiobooks(db_pool).await.map(MetaResponse::Count)
                }
            }
        })
        .await
}

async fn execute_id_name_query<T, F>(
    db_pool: &PgPool,
    query: &str,
    page: u32,
    limit: u32,
    mapper: F,
) -> Result<Vec<T>, AppError>
where
    F: Fn(IdNameRow) -> T, {
    let (limit, offset) = get_limit_offset(page, limit);
    let rows = sqlx::query_as::<_, IdNameRow>(query)
        .bind(limit)
        .bind(offset)
        .fetch_all(db_pool)
        .await
        .map_err(|e| AppError::DeserializationError(format!("Database query failed: {e}")))?;

    Ok(rows.into_iter().map(mapper).collect())
}

async fn execute_id_title_query<T, F>(
    db_pool: &PgPool,
    query: &str,
    page: u32,
    limit: u32,
    mapper: F,
) -> Result<Vec<T>, AppError>
where
    F: Fn(IdTitleRow) -> T, {
    let (limit, offset) = get_limit_offset(page, limit);
    let rows = sqlx::query_as::<_, IdTitleRow>(query)
        .bind(limit)
        .bind(offset)
        .fetch_all(db_pool)
        .await
        .map_err(|e| AppError::DeserializationError(format!("Database query failed: {e}")))?;

    Ok(rows.into_iter().map(mapper).collect())
}

// --- Published Audiobooks Implementations ---

async fn get_categories_by_published_audiobooks(
    pool: &PgPool,
    page: u32,
    limit: u32,
) -> Result<Vec<Category>, AppError> {
    let query = "
        SELECT c.id, c.name
        FROM category c
        LEFT JOIN audiobook_category abc ON c.id = abc.category_id
        GROUP BY c.id, c.name
        ORDER BY COUNT(abc.audiobook_id) DESC, c.name ASC
        LIMIT $1 OFFSET $2
    ";
    execute_id_name_query(pool, query, page, limit, |r| Category {
        id: r.id,
        value: r.name,
    })
    .await
}

async fn get_keywords_by_published_audiobooks(
    pool: &PgPool,
    page: u32,
    limit: u32,
) -> Result<Vec<Keyword>, AppError> {
    let query = "
        SELECT k.id, k.name
        FROM keyword k
        LEFT JOIN audiobook_keyword abk ON k.id = abk.keyword_id
        GROUP BY k.id, k.name
        ORDER BY COUNT(abk.audiobook_id) DESC, k.name ASC
        LIMIT $1 OFFSET $2
    ";
    execute_id_name_query(pool, query, page, limit, |r| Keyword {
        id: r.id,
        value: r.name,
    })
    .await
}

async fn get_authors_by_published_audiobooks(
    pool: &PgPool,
    page: u32,
    limit: u32,
) -> Result<Vec<Author>, AppError> {
    let query = "
        SELECT a.id, a.name
        FROM author a
        LEFT JOIN audiobook_author aba ON a.id = aba.author_id
        GROUP BY a.id, a.name
        ORDER BY COUNT(aba.audiobook_id) DESC, a.name ASC
        LIMIT $1 OFFSET $2
    ";
    execute_id_name_query(pool, query, page, limit, |r| Author {
        id: r.id,
        name: r.name,
    })
    .await
}

async fn get_readers_by_published_audiobooks(
    pool: &PgPool,
    page: u32,
    limit: u32,
) -> Result<Vec<Reader>, AppError> {
    let query = "
        SELECT r.id, r.name
        FROM reader r
        LEFT JOIN audiobook_reader abr ON r.id = abr.reader_id
        GROUP BY r.id, r.name
        ORDER BY COUNT(abr.audiobook_id) DESC, r.name ASC
        LIMIT $1 OFFSET $2
    ";
    execute_id_name_query(pool, query, page, limit, |r| Reader {
        id: r.id,
        name: r.name,
    })
    .await
}

async fn get_series_by_published_audiobooks(
    pool: &PgPool,
    page: u32,
    limit: u32,
) -> Result<Vec<Series>, AppError> {
    let query = "
        SELECT s.id, s.title
        FROM series s
        LEFT JOIN audiobook a ON s.id = a.series_id
        GROUP BY s.id, s.title
        ORDER BY COUNT(a.id) DESC, s.title ASC
        LIMIT $1 OFFSET $2
    ";
    execute_id_title_query(pool, query, page, limit, |r| Series {
        id: r.id,
        title: r.title,
    })
    .await
}

// --- N Subscribers Implementations ---

async fn get_categories_by_n_subscribers(
    pool: &PgPool,
    page: u32,
    limit: u32,
) -> Result<Vec<Category>, AppError> {
    let query = "
        SELECT c.id, c.name
        FROM category c
        LEFT JOIN user_category_notification ucn ON c.id = ucn.category_id
        GROUP BY c.id, c.name
        ORDER BY COUNT(ucn.user_id) DESC, c.name ASC
        LIMIT $1 OFFSET $2
    ";
    execute_id_name_query(pool, query, page, limit, |r| Category {
        id: r.id,
        value: r.name,
    })
    .await
}

async fn get_keywords_by_n_subscribers(
    pool: &PgPool,
    page: u32,
    limit: u32,
) -> Result<Vec<Keyword>, AppError> {
    let query = "
        SELECT k.id, k.name
        FROM keyword k
        LEFT JOIN user_keyword_notification ukn ON k.id = ukn.keyword_id
        GROUP BY k.id, k.name
        ORDER BY COUNT(ukn.user_id) DESC, k.name ASC
        LIMIT $1 OFFSET $2
    ";
    execute_id_name_query(pool, query, page, limit, |r| Keyword {
        id: r.id,
        value: r.name,
    })
    .await
}

async fn get_authors_by_n_subscribers(
    pool: &PgPool,
    page: u32,
    limit: u32,
) -> Result<Vec<Author>, AppError> {
    let query = "
        SELECT a.id, a.name
        FROM author a
        LEFT JOIN user_author_notification uan ON a.id = uan.author_id
        GROUP BY a.id, a.name
        ORDER BY COUNT(uan.user_id) DESC, a.name ASC
        LIMIT $1 OFFSET $2
    ";
    execute_id_name_query(pool, query, page, limit, |r| Author {
        id: r.id,
        name: r.name,
    })
    .await
}

async fn get_readers_by_n_subscribers(
    pool: &PgPool,
    page: u32,
    limit: u32,
) -> Result<Vec<Reader>, AppError> {
    let query = "
        SELECT r.id, r.name
        FROM reader r
        LEFT JOIN user_reader_notification urn ON r.id = urn.reader_id
        GROUP BY r.id, r.name
        ORDER BY COUNT(urn.user_id) DESC, r.name ASC
        LIMIT $1 OFFSET $2
    ";
    execute_id_name_query(pool, query, page, limit, |r| Reader {
        id: r.id,
        name: r.name,
    })
    .await
}

async fn get_series_by_n_subscribers(
    pool: &PgPool,
    page: u32,
    limit: u32,
) -> Result<Vec<Series>, AppError> {
    let query = "
        SELECT s.id, s.title
        FROM series s
        LEFT JOIN user_series_notification usn ON s.id = usn.series_id
        GROUP BY s.id, s.title
        ORDER BY COUNT(usn.user_id) DESC, s.title ASC
        LIMIT $1 OFFSET $2
    ";
    execute_id_title_query(pool, query, page, limit, |r| Series {
        id: r.id,
        title: r.title,
    })
    .await
}

async fn get_categories_alphabetically(
    pool: &PgPool,
    page: u32,
    limit: u32,
) -> Result<Vec<Category>, AppError> {
    let query = "
        SELECT id, name
        FROM category
        ORDER BY name ASC
        LIMIT $1 OFFSET $2
    ";
    execute_id_name_query(pool, query, page, limit, |r| Category {
        id: r.id,
        value: r.name,
    })
    .await
}

async fn get_keywords_alphabetically(
    pool: &PgPool,
    page: u32,
    limit: u32,
) -> Result<Vec<Keyword>, AppError> {
    let query = "
        SELECT id, name
        FROM keyword
        ORDER BY name ASC
        LIMIT $1 OFFSET $2
    ";
    execute_id_name_query(pool, query, page, limit, |r| Keyword {
        id: r.id,
        value: r.name,
    })
    .await
}

async fn get_authors_alphabetically(
    pool: &PgPool,
    page: u32,
    limit: u32,
) -> Result<Vec<Author>, AppError> {
    let query = "
        SELECT id, name
        FROM author
        ORDER BY name ASC
        LIMIT $1 OFFSET $2
    ";
    execute_id_name_query(pool, query, page, limit, |r| Author {
        id: r.id,
        name: r.name,
    })
    .await
}

async fn get_readers_alphabetically(
    pool: &PgPool,
    page: u32,
    limit: u32,
) -> Result<Vec<Reader>, AppError> {
    let query = "
        SELECT id, name
        FROM reader
        ORDER BY name ASC
        LIMIT $1 OFFSET $2
    ";
    execute_id_name_query(pool, query, page, limit, |r| Reader {
        id: r.id,
        name: r.name,
    })
    .await
}

async fn get_series_alphabetically(
    pool: &PgPool,
    page: u32,
    limit: u32,
) -> Result<Vec<Series>, AppError> {
    let query = "
        SELECT id, title
        FROM series
        ORDER BY title ASC
        LIMIT $1 OFFSET $2
    ";
    execute_id_title_query(pool, query, page, limit, |r| Series {
        id: r.id,
        title: r.title,
    })
    .await
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
async fn count_audiobooks_for_category(pool: &PgPool, category_id: i64) -> Result<u32, AppError> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audiobook_category WHERE category_id = $1")
            .bind(category_id)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::DeserializationError(format!("Database query failed: {e}")))?;
    Ok(count as u32)
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
async fn count_audiobooks_for_keyword(pool: &PgPool, keyword_id: i64) -> Result<u32, AppError> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audiobook_keyword WHERE keyword_id = $1")
            .bind(keyword_id)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::DeserializationError(format!("Database query failed: {e}")))?;
    Ok(count as u32)
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
async fn count_audiobooks_for_author(pool: &PgPool, author_id: i64) -> Result<u32, AppError> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audiobook_author WHERE author_id = $1")
            .bind(author_id)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::DeserializationError(format!("Database query failed: {e}")))?;
    Ok(count as u32)
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
async fn count_audiobooks_for_reader(pool: &PgPool, reader_id: i64) -> Result<u32, AppError> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audiobook_reader WHERE reader_id = $1")
            .bind(reader_id)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::DeserializationError(format!("Database query failed: {e}")))?;
    Ok(count as u32)
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
async fn count_audiobooks_in_series(pool: &PgPool, series_id: i64) -> Result<u32, AppError> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audiobook WHERE series_id = $1")
        .bind(series_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DeserializationError(format!("Database query failed: {e}")))?;
    Ok(count as u32)
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
async fn count_all_audiobooks(pool: &PgPool) -> Result<u32, AppError> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audiobook")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DeserializationError(format!("Database query failed: {e}")))?;
    Ok(count as u32)
}
