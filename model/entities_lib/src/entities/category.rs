//! Category stuff

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Category {
    pub id: i64,
    pub value: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct CategoryWithMetadata {
    pub category: Category,
    pub n_audiobooks: u32,
}
