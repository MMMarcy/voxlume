// pub mod db;
pub mod entities;
pub mod error;
pub use self::error::{Error, Result};

pub use entities::audiobook::AudioBook;
pub use entities::audiobook_requests::GetAudioBookRequestType;
pub use entities::author::Author;
pub use entities::category::Category;
pub use entities::enums::Language;
pub use entities::keyword::Keyword;
pub use entities::meta_request::{MetaRequest, MetaResponse};
pub use entities::notifications::{NotificationReason, UserNotification};
pub use entities::reader::Reader;
pub use entities::search_query::SearchQuery;
pub use entities::series::Series;
pub use entities::shareable_args::{Environment, ShareableArgsValues};
pub use entities::subscription::{SubscriptionExists, SubscriptionType};
pub use entities::user::User;

pub type AudiobookWithData = (
    AudioBook,
    Vec<Author>,
    Vec<Category>,
    Vec<Keyword>,
    Vec<Reader>,
    Option<Series>,
);
