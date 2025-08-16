use entities_lib::{
    AudioBook, AudiobookWithData, Author, Category, GetAudioBookRequestType, Keyword, Reader,
    Series,
};
use neo4rs::{query, Error as Neo4rsError, Graph};
use tracing::{instrument, trace};

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

/// .
///
/// # Errors
///
/// This function will return an error if.
#[instrument(skip_all)]
pub async fn get_audiobooks_cached(
    graph: &Graph,
    cache: &Cache<GetAudioBookRequestType, Result<Vec<AudiobookWithData>, AppError>>,
    request_type: GetAudioBookRequestType,
    limit: u16,
) -> Result<Vec<AudiobookWithData>, AppError> {
    let cache_key = request_type.clone();
    let res = match request_type {
        GetAudioBookRequestType::MostRecent(page) => {
            cache
                .get_with(cache_key, async {
                    get_most_recent_audiobooks_with_data(graph, limit, page).await
                })
                .await
        }
        GetAudioBookRequestType::ByAuthor(author, page) => {
            cache
                .get_with(cache_key, async {
                    get_audiobooks_with_data_by_author(graph, author, limit, page).await
                })
                .await
        }
        GetAudioBookRequestType::ByReader(reader, page) => {
            cache
                .get_with(cache_key, async {
                    get_audiobooks_with_data_by_reader(graph, reader, limit, page).await
                })
                .await
        }
        GetAudioBookRequestType::ByCategory(category, page) => {
            cache
                .get_with(cache_key, async {
                    get_audiobooks_with_data_by_category(graph, category, limit, page).await
                })
                .await
        }
        GetAudioBookRequestType::ByKeyword(keyword, page) => {
            cache
                .get_with(cache_key, async {
                    get_audiobooks_with_data_by_keyword(graph, keyword, limit, page).await
                })
                .await
        }
        GetAudioBookRequestType::BySeries(series, page) => {
            cache
                .get_with(cache_key, async {
                    get_audiobooks_with_data_by_series(graph, series, limit, page).await
                })
                .await
        }
    };
    res
}

#[instrument(skip_all)]
async fn get_audiobooks_with_data_by_category(
    graph: &Graph,
    category: Category,
    limit: u16,
    _page: u16,
) -> Result<Vec<AudiobookWithData>, AppError> {
    let get_audiobook_with_connections_query = query(
        "
        MATCH (target_category:Category {value: $category_value})
        MATCH (target_category)<-[:CATEGORIZED_AS]-(ab:Audiobook)
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
    .param("category_value", category.value)
    .param("limit", limit);

    let mut result_stream = graph.execute(get_audiobook_with_connections_query).await?;
    let mut audiobooks_data: Vec<AudiobookWithData> = Vec::new();

    while let Ok(maybe_row) = result_stream.next().await {
        if let Some(row) = maybe_row {
            let audiobook: AudioBook = row.get("audiobook").map_err(|e| {
                AppError::DeserializationError(format!("Failed to get audiobook: {e}"))
            })?;
            let authors: Vec<Author> = row.get("authors").map_err(|e| {
                AppError::DeserializationError(format!("Failed to get authors: {e}"))
            })?;
            let categories: Vec<Category> = row.get("categories").map_err(|e| {
                AppError::DeserializationError(format!("Failed to get categories: {e}"))
            })?;
            let keywords: Vec<Keyword> = row.get("keywords").map_err(|e| {
                AppError::DeserializationError(format!("Failed to get keywords: {e}"))
            })?;
            let readers: Vec<Reader> = row.get("readers").map_err(|e| {
                AppError::DeserializationError(format!("Failed to get readers: {e}"))
            })?;
            let series: Option<Series> = row.get("series").map_err(|e| {
                AppError::DeserializationError(format!("Failed to get series: {e}"))
            })?;

            audiobooks_data.push((audiobook, authors, categories, keywords, readers, series));
        } else {
            break;
        }
    }

    Ok(audiobooks_data)
}

#[instrument(skip_all)]
async fn get_audiobooks_with_data_by_keyword(
    graph: &Graph,
    keyword: Keyword,
    limit: u16,
    _page: u16,
) -> Result<Vec<AudiobookWithData>, AppError> {
    let get_audiobook_with_connections_query = query(
        "
        MATCH (target_keyword:Keyword {value: $keyword_value})
        MATCH (target_keyword)<-[:HAS_KEYWORD]-(ab:Audiobook)
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
    .param("keyword_value", keyword.value)
    .param("limit", limit);

    let mut result_stream = graph.execute(get_audiobook_with_connections_query).await?;
    let mut audiobooks_data: Vec<AudiobookWithData> = Vec::new();

    while let Ok(maybe_row) = result_stream.next().await {
        if let Some(row) = maybe_row {
            let audiobook: AudioBook = row.get("audiobook").map_err(|e| {
                AppError::DeserializationError(format!("Failed to get audiobook: {}", e))
            })?;
            let authors: Vec<Author> = row.get("authors").map_err(|e| {
                AppError::DeserializationError(format!("Failed to get authors: {}", e))
            })?;
            let categories: Vec<Category> = row.get("categories").map_err(|e| {
                AppError::DeserializationError(format!("Failed to get categories: {}", e))
            })?;
            let keywords: Vec<Keyword> = row.get("keywords").map_err(|e| {
                AppError::DeserializationError(format!("Failed to get keywords: {}", e))
            })?;
            let readers: Vec<Reader> = row.get("readers").map_err(|e| {
                AppError::DeserializationError(format!("Failed to get readers: {}", e))
            })?;
            let series: Option<Series> = row.get("series").map_err(|e| {
                AppError::DeserializationError(format!("Failed to get series: {}", e))
            })?;

            audiobooks_data.push((audiobook, authors, categories, keywords, readers, series));
        } else {
            break;
        }
    }

    Ok(audiobooks_data)
}

#[instrument(skip_all)]
async fn get_audiobooks_with_data_by_series(
    graph: &Graph,
    series: Series,
    limit: u16,
    _page: u16,
) -> Result<Vec<AudiobookWithData>, AppError> {
    let get_audiobook_with_connections_query = query(
        "
        MATCH (target_series:Series {title: $series_title})
        MATCH (target_series)<-[:PART_OF_SERIES]-(ab:Audiobook)
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

        OPTIONAL MATCH (ab)-[:PART_OF_SERIES]->(series_node:Series)

        RETURN ab AS audiobook, authors, categories, keywords, readers, series_node as series
    ",
    )
    .param("series_title", series.title)
    .param("limit", limit);

    let mut result_stream = graph.execute(get_audiobook_with_connections_query).await?;
    let mut audiobooks_data: Vec<AudiobookWithData> = Vec::new();

    while let Ok(maybe_row) = result_stream.next().await {
        if let Some(row) = maybe_row {
            let audiobook: AudioBook = row.get("audiobook").map_err(|e| {
                AppError::DeserializationError(format!("Failed to get audiobook: {e}"))
            })?;
            let authors: Vec<Author> = row.get("authors").map_err(|e| {
                AppError::DeserializationError(format!("Failed to get authors: {e}"))
            })?;
            let categories: Vec<Category> = row.get("categories").map_err(|e| {
                AppError::DeserializationError(format!("Failed to get categories: {e}"))
            })?;
            let keywords: Vec<Keyword> = row.get("keywords").map_err(|e| {
                AppError::DeserializationError(format!("Failed to get keywords: {}", e))
            })?;
            let readers: Vec<Reader> = row.get("readers").map_err(|e| {
                AppError::DeserializationError(format!("Failed to get readers: {}", e))
            })?;
            let series: Option<Series> = row.get("series").map_err(|e| {
                AppError::DeserializationError(format!("Failed to get series: {}", e))
            })?;

            audiobooks_data.push((audiobook, authors, categories, keywords, readers, series));
        } else {
            break;
        }
    }

    Ok(audiobooks_data)
}

#[instrument(skip_all)]
async fn get_audiobooks_with_data_by_reader(
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
async fn get_audiobooks_with_data_by_author(
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
            .map_err(|e| AppError::DeserializationError(format!("Failed to get series: {e}")))?;

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
        let audiobook: AudioBook = row
            .get("audiobook")
            .map_err(|e| AppError::DeserializationError(format!("Failed to get audiobook: {e}")))?;

        // neo4rs can deserialize a list of nodes into a Vec<Struct>
        let authors: Vec<Author> = row
            .get("authors")
            .map_err(|e| AppError::DeserializationError(format!("Failed to get authors: {e}")))?;
        let categories: Vec<Category> = row.get("categories").map_err(|e| {
            AppError::DeserializationError(format!("Failed to get categories: {e}"))
        })?;
        let keywords: Vec<Keyword> = row
            .get("keywords")
            .map_err(|e| AppError::DeserializationError(format!("Failed to get keywords: {e}")))?;
        let readers: Vec<Reader> = row
            .get("readers")
            .map_err(|e| AppError::DeserializationError(format!("Failed to get readers: {e}")))?;

        // For Option<Series>, if 'series' is null in the DB, .get() should produce a None variant correctly.
        let series: Option<Series> = row
            .get("series")
            .map_err(|e| AppError::DeserializationError(format!("Failed to get series: {e}")))?;

        audiobooks_data.push((audiobook, authors, categories, keywords, readers, series));
    }

    Ok(audiobooks_data)
}
