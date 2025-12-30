use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SearchQuery {
    pub search_string: String,
}

// #[derive(Debug, Hash, Serialize, Deserialize)]
// pub struct EmbeddedSearchQuery {
//     pub search_string: String,
//     pub embeddings: [f32; 256],
// }
