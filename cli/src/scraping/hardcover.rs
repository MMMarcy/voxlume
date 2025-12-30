use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HardcoverBook {
    pub id: i64,
    pub title: String,
    pub rating: Option<f64>,
    pub ratings_count: Option<i64>,
    pub ratings_distribution: Option<serde_json::Value>,
    pub slug: Option<String>,
    pub reviews_count: Option<i64>,
}

#[derive(Deserialize, Debug)]
struct HardcoverSearchResponse {
    data: HardcoverSearchData,
}

#[derive(Deserialize, Debug)]
struct HardcoverSearchData {
    search: HardcoverSearchRoot,
}

#[derive(Deserialize, Debug)]
struct HardcoverSearchRoot {
    results: HardcoverSearchResults,
}

#[derive(Deserialize, Debug)]
struct HardcoverSearchResults {
    hits: Vec<HardcoverSearchHit>,
}

#[derive(Deserialize, Debug)]
struct HardcoverSearchHit {
    document: HardcoverSearchDocument,
}

#[derive(Deserialize, Debug)]
struct HardcoverSearchDocument {
    id: String,
    title: String,
    rating: Option<f64>,
    ratings_count: Option<i64>,
    slug: Option<String>,
    reviews_count: Option<i64>,
}

/// # Errors
/// If there are errors with the graphql request.
pub async fn search_book(
    client: &Client,
    api_key: &str,
    title: &str,
    author: &str,
) -> Result<Option<HardcoverBook>, Box<dyn Error>> {
    let query = r#"
        query SearchBooks($query: String!) {
          search(
            query: $query
            query_type: "Audiobook"
            per_page: 1
            page: 1
          ) {
            results
          }
        }
    "#;

    let search_query = format!("{title} {author}");
    let body = json!({
        "query": query,
        "variables": {
            "query": search_query
        }
    });

    let response = client
        .post("https://api.hardcover.app/v1/graphql")
        .header("content-type", "application/json")
        .header("Authorization", api_key)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Hardcover API error: {}", response.status()).into());
    }

    let response_body: HardcoverSearchResponse = response.json().await?;

    if let Some(hit) = response_body.data.search.results.hits.into_iter().next() {
        let doc = hit.document;
        Ok(Some(HardcoverBook {
            id: doc.id.parse()?,
            title: doc.title,
            rating: doc.rating,
            ratings_count: doc.ratings_count,
            ratings_distribution: None,
            slug: doc.slug,
            reviews_count: doc.reviews_count,
        }))
    } else {
        Ok(None)
    }
}
