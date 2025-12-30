use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardcoverAudiobookMetadata {
    pub id: i64,
    pub audiobook_id: i64,
    pub metadata: serde_json::Value,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardcoverAuthorMetadata {
    pub id: i64,
    pub author_id: i64,
    pub metadata: serde_json::Value,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardcoverSeriesMetadata {
    pub id: i64,
    pub series_id: i64,
    pub metadata: serde_json::Value,
}
