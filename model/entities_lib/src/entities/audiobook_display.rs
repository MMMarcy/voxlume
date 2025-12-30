use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize, Default)]
pub enum AudiobookDisplayMode {
    #[default]
    TableLike,
    ListLike,
    GridLike,
}

#[derive(Debug, PartialEq)]
pub struct ParseDisplayModeError;

impl FromStr for AudiobookDisplayMode {
    type Err = ParseDisplayModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tablelike" => Ok(AudiobookDisplayMode::TableLike),
            "listlike" => Ok(AudiobookDisplayMode::ListLike),
            "gridlike" => Ok(AudiobookDisplayMode::GridLike),
            // For any other string, return an error
            _ => Err(ParseDisplayModeError),
        }
    }
}

impl Display for AudiobookDisplayMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudiobookDisplayMode::TableLike => write!(f, "tablelike"),
            AudiobookDisplayMode::ListLike => write!(f, "listlike"),
            AudiobookDisplayMode::GridLike => write!(f, "gridlike"),
        }
    }
}
