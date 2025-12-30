use std::fmt::Display;

pub mod parade;
pub mod pgmq;

#[derive(Debug, Clone)]
pub enum AppError {
    Neo4jError(String),
    DeserializationError(String), // Or use a more specific error type
    NoAuthorProvided,
    NoReaderProvided,
    GenericError(String),
    NotFound(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: String = match self {
            AppError::NoAuthorProvided => "No author provided".to_string(),
            AppError::NoReaderProvided => "No reader provided".to_string(),
            AppError::Neo4jError(s)
            | AppError::DeserializationError(s)
            | AppError::GenericError(s)
            | AppError::NotFound(s) => s.clone(),
        };
        write!(f, "{value}")
    }
}
