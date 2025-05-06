// pub mod db;
pub mod entities;
pub mod error;
pub use self::error::{Error, Result};

pub use entities::audiobook::AudioBook;
pub use entities::author::Author;
pub use entities::category::Category;
pub use entities::enums::Language;
pub use entities::keyword::Keyword;
pub use entities::reader::Reader;
pub use entities::series::Series;
pub use entities::user::User;

pub type AudiobookWithData = (
    AudioBook,
    Vec<Author>,
    Vec<Category>,
    Vec<Keyword>,
    Vec<Reader>,
    Option<Series>,
);
