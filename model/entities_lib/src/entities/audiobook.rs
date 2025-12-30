#![allow(clippy::doc_markdown)]
#![allow(clippy::used_underscore_items)]

//! Module defining a book.
// use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioBook {
    pub id: i64,

    pub title: String,

    pub bitrate: Option<String>,

    pub categories: Vec<String>,

    pub cover_url: Option<String>,

    pub description: String,

    pub very_short_description: String,

    pub description_for_embeddings: String,

    pub file_size: Option<String>,

    pub format: Option<String>,

    pub keywords: Vec<String>,

    pub language: String,

    /// Path where the audiobook is stored on audiobookbay.
    pub path: String,

    /// The timestamp when the change list item was created.
    pub last_upload: i64,

    /// Title
    pub unabriged: bool,

    pub series_volume: Option<String>,
}
