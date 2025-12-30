//! Search operations on Parade.

use std::collections::HashMap;
use std::sync::Arc;

use entities_lib::SearchQuery;
use sqlx::postgres::PgRow;
use sqlx::{self, PgPool, Row};
use tracing::{debug, instrument};

use crate::db_ops::AppError;
use crate::utils::gemini::GeminiContentEmbedderLike;
use entities_lib::ShareableArgsValues;

const VECTOR_QUERY: &str = r"
SELECT
    id,
    (optimized_description_embedding <#> ($1)::vector) * -1 AS score
FROM audiobook
ORDER BY (optimized_description_embedding <#> ($1)::vector) ASC
LIMIT $2
";

const BM25_QUERY: &str = r"
SELECT
    audiobook_id AS id,
    paradedb.score(audiobook_id) AS score
FROM audiobook_search_view
WHERE search_content @@@ $1
ORDER BY score DESC
LIMIT $2
";

/// # Errors
///
/// .
///
/// # Panics
///
/// .
#[allow(clippy::cast_sign_loss)]
#[instrument(skip_all)]
pub async fn search_audiobooks(
    pool: &PgPool,
    search_query: &SearchQuery,
    shareable_args: &ShareableArgsValues,
    embedder: Arc<dyn GeminiContentEmbedderLike>,
) -> Result<Vec<i64>, AppError> {
    let embeddings = embedder
        .embed_content(
            &search_query.search_string,
            &gemini_rust::TaskType::RetrievalDocument,
            shareable_args.gemini_embeddings_size,
        )
        .await
        .map_err(|e| AppError::GenericError(e.to_string()))?;

    // Run Vector Search
    let vector_future = sqlx::query(VECTOR_QUERY)
        .bind(embeddings) // Note: ensure embeddings is cloned or cheap to reference if needed
        .bind(shareable_args.max_search_results)
        .map(|row: PgRow| {
            (
                row.try_get::<i64, _>("id").unwrap(),
                row.try_get::<f64, _>("score").unwrap(),
            )
        })
        .fetch_all(pool);

    let bm25_future = sqlx::query(BM25_QUERY)
        .bind(&search_query.search_string)
        .bind(shareable_args.max_search_results)
        .map(|row: PgRow| {
            (
                row.try_get::<i64, _>("id").unwrap(),
                f64::from(row.try_get::<f32, _>("score").unwrap()),
            )
        })
        .fetch_all(pool);

    // 2. Run them concurrently
    let (vector_results_res, bm25_results_res) = tokio::join!(vector_future, bm25_future);

    // 3. Handle errors
    let vector_results = vector_results_res.map_err(|e| AppError::GenericError(e.to_string()))?;
    let bm25_results = bm25_results_res.map_err(|e| AppError::GenericError(e.to_string()))?;
    debug!("Semantic results: {:?}", &vector_results);
    debug!("BM25 results: {:?}", &bm25_results);

    Ok(combine_results_hybrid(vector_results, bm25_results))
}

#[allow(clippy::cast_precision_loss)]
fn combine_results_hybrid(
    vector_results: Vec<(i64, f64)>,
    bm25_results: Vec<(i64, f64)>,
) -> Vec<i64> {
    let mut scores: HashMap<i64, f64> = HashMap::new();
    let rrf_k = 60.0;

    let mut process_results = |results: Vec<(i64, f64)>| {
        for (rank, (id, _)) in results.into_iter().enumerate() {
            // Reciprocal Rank Fusion (RRF)
            // score = 1 / (k + rank)
            // We use rank + 1.0 because rank is 0-indexed
            let score = 1.0 / (rrf_k + (rank as f64) + 1.0);
            *scores.entry(id).or_default() += score;
        }
    };

    process_results(vector_results);
    process_results(bm25_results);

    let mut result: Vec<(i64, f64)> = scores.into_iter().collect();
    // Sort by score descending
    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let res = result.into_iter().map(|(id, _)| id).collect();
    debug!("Final results {:?}", &res);
    res
}

/// Checks if an audiobook with the given path already exists.
///
/// # Errors
///
/// Returns an error if the database query fails.
#[instrument(skip(pool))]
pub async fn does_audiobook_exists(pool: &PgPool, path: &str) -> Result<bool, AppError> {
    sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM audiobook WHERE path = $1)")
        .bind(path)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::GenericError(e.to_string()))
}
