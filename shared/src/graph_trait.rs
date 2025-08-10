use entities_lib::{
    entities::user::GUEST_USER_ID, AudioBook, AudiobookWithData, Author, Category,
    GetAudioBookRequestType, Keyword, Reader, Series,
};
use neo4rs::{query, Error as Neo4rsError, Graph};
use tracing::{error, instrument, trace};

use moka::future::Cache;

#[derive(Debug, Clone)]
pub enum AppError {
    Neo4jError(String),
    DeserializationError(String), // Or use a more specific error type
    NoAuthorProvided,
    NoReaderProvided,
}

impl From<Neo4rsError> for AppError {
    fn from(err: Neo4rsError) -> Self {
        AppError::Neo4jError(err.to_string())
    }
}

#[instrument(skip_all)]
pub async fn get_audiobooks_cached(
    graph: &Graph,
    cache: &Cache<
        (i64, GetAudioBookRequestType, u16, u16),
        Result<Vec<AudiobookWithData>, AppError>,
    >,
    maybe_user_id: Option<i64>,
    request_type: GetAudioBookRequestType,
    limit: u16,
    page: u16,
) -> Result<Vec<AudiobookWithData>, AppError> {
    let user_id = maybe_user_id.unwrap_or(GUEST_USER_ID);
    let cache_key = (user_id, request_type.clone(), limit, page);
    let res = match request_type {
        GetAudioBookRequestType::MostRecent => {
            cache
                .get_with(cache_key, async {
                    get_most_recent_audiobooks_with_data(graph, limit, page).await
                })
                .await
        }
        GetAudioBookRequestType::ByAuthor(author) => {
            get_audiobooks_with_data_by_author(graph, author, limit, page).await
        }
        GetAudioBookRequestType::ByReader(reader) => {
            get_audiobooks_with_data_by_reader(graph, reader, limit, page).await
        }
        GetAudioBookRequestType::ByCategory(category) => todo!(),
        GetAudioBookRequestType::ByKeyword(series) => todo!(),
        GetAudioBookRequestType::BySeries(series) => todo!(),
    };
    res
}

#[instrument(skip_all)]
pub async fn get_audiobooks_with_data_by_reader(
    graph: &Graph,
    reader: Reader,
    limit: u16,
    _page: u16,
) -> Result<Vec<AudiobookWithData>, AppError> {
    let get_audiobook_with_connections_query = query(
        "
        MATCH (target_reader:Reader {name: $reader_name})
        MATCH (target_reader)<-[:READ_BY]-(ab:Audiobook)
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
    .param("reader_name", reader.name)
    .param("limit", limit);

    let mut result_stream = graph.execute(get_audiobook_with_connections_query).await?;
    let mut audiobooks_data: Vec<AudiobookWithData> = Vec::new();

    while let Ok(maybe_row) = result_stream.next().await {
        trace!("Inside getting row. Row: {:?}", maybe_row);
        if maybe_row.is_none() {
            break;
        }
        let row = maybe_row.expect("Should be ok to extract the value");

        let audiobook: AudioBook = row.get("audiobook").map_err(|e| {
            AppError::DeserializationError(format!("Failed to get audiobook: {}", e))
        })?;

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

        let series: Option<Series> = row
            .get("series")
            .map_err(|e| AppError::DeserializationError(format!("Failed to get series: {}", e)))?;

        audiobooks_data.push((audiobook, authors, categories, keywords, readers, series));
    }

    Ok(audiobooks_data)
}

#[instrument(skip_all)]
pub async fn get_audiobooks_with_data_by_author(
    graph: &Graph,
    author: Author,
    limit: u16,
    _page: u16,
) -> Result<Vec<AudiobookWithData>, AppError> {
    let get_audiobook_with_connections_query = query(
        "
        MATCH (target_author:Author {name: $author_name})
        MATCH (target_author)<-[:WRITTEN_BY]-(ab:Audiobook)
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
    .param("author_name", author.name)
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
