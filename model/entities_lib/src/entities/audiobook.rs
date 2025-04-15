#![allow(clippy::doc_markdown)]
#![allow(clippy::used_underscore_items)]

//! Module defining a book.
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioBook {
    pub bitrate: Option<String>,

    pub categories: Vec<String>,

    pub cover_url: String,

    pub description: String,

    pub file_size: String,

    pub format: String,

    pub keywords: Vec<String>,

    pub language: String,

    /// Path where the audiobook is stored on audiobookbay.
    pub path: String,

    /// The timestamp when the change list item was created.
    pub last_upload: NaiveDateTime,

    /// Title
    pub title: String,

    pub unabriged: bool,

    pub series_volume: Option<String>,
}
