use entities_lib::{
    AudioBook, AudiobookWithData, Author, Category, GetAudioBookRequestType, Keyword, Reader,
    Series, User,
};
use neo4rs::{query, Error as Neo4rsError, Graph, Row}; // Make sure to import necessary items
use serde::Deserialize; // Already shown above
use tracing::{info, instrument, trace};

use crate::state::AppState;

#[derive(Debug, Clone)]
pub enum AppError {
    Neo4jError(String),
    DeserializationError(String), // Or use a more specific error type
}

impl From<Neo4rsError> for AppError {
    fn from(err: Neo4rsError) -> Self {
        AppError::Neo4jError(err.to_string())
    }
}

#[instrument]
pub async fn get_audiobooks_cached(
    app_state: AppState,
    user_id: i64,
    request_type: GetAudioBookRequestType,
    limit: u16,
    page: u16,
) -> Result<Vec<AudiobookWithData>, AppError> {
    let cache = app_state.cache;
    let cache_key = (user_id, request_type, limit, page);
    let res = match request_type {
        GetAudioBookRequestType::MostRecent => {
            cache
                .get_with(cache_key, async {
                    get_most_recent_audiobooks_with_data(&app_state.graph, limit, page).await
                })
                .await
        }
    };
    res
}

#[instrument(skip_all)]
async fn get_most_recent_audiobooks_with_data(
    graph: &Graph,
    limit: u16,
    _page: u16,
) -> Result<Vec<AudiobookWithData>, AppError> {
    let get_audiobook_with_connections_query = query(
        "
        MATCH (ab:Audiobook)
        WITH ab
        ORDER BY ab.last_upload DESC
        LIMIT $limit

        OPTIONAL MATCH (ab)-[:WRITTEN_BY]->(author:Author)
        WITH ab, collect(author) AS authors

        OPTIONAL MATCH (ab)-[:CATEGORIZED_AS]->(category:Category)
        WITH ab, authors, collect(category) AS categories

        OPTIONAL MATCH (ab)-[:HAS_KEYWORD]->(keyword:Keyword)
        WITH ab, authors, categories, collect(keyword) AS keywords

        OPTIONAL MATCH (ab)-[:READ_BY]->(reader:Reader)
        WITH ab, authors, categories, keywords, collect(reader) AS readers

        OPTIONAL MATCH (ab)-[:PART_OF_SERIES]->(series:Series)

        RETURN ab AS audiobook, authors, categories, keywords, readers, series
    ",
    )
    .param("limit", limit);

    let mut result_stream = graph.execute(get_audiobook_with_connections_query).await?;
    let mut audiobooks_data: Vec<AudiobookWithData> = Vec::new();

    while let Ok(maybe_row) = result_stream.next().await {
        trace!("Inside getting row. Row: {:?}", maybe_row);
        if maybe_row.is_none() {
            break;
        }
        let row = maybe_row.expect("Should be ok to extract the value");

        // neo4rs can deserialize node properties into a struct
        let audiobook: AudioBook = row.get("audiobook").map_err(|e| {
            AppError::DeserializationError(format!("Failed to get audiobook: {}", e))
        })?;

        // neo4rs can deserialize a list of nodes into a Vec<Struct>
        let authors: Vec<Author> = row
            .get("authors")
            .map_err(|e| AppError::DeserializationError(format!("Failed to get authors: {}", e)))?;
        let categories: Vec<Category> = row.get("categories").map_err(|e| {
            AppError::DeserializationError(format!("Failed to get categories: {}", e))
        })?;
        let keywords: Vec<Keyword> = row.get("keywords").map_err(|e| {
            AppError::DeserializationError(format!("Failed to get keywords: {}", e))
        })?;
        let readers: Vec<Reader> = row
            .get("readers")
            .map_err(|e| AppError::DeserializationError(format!("Failed to get readers: {}", e)))?;

        // For Option<Series>, if 'series' is null in the DB, .get() should produce a None variant correctly.
        let series: Option<Series> = row
            .get("series")
            .map_err(|e| AppError::DeserializationError(format!("Failed to get series: {}", e)))?;

        audiobooks_data.push((audiobook, authors, categories, keywords, readers, series));
    }

    Ok(audiobooks_data)
}
