#![allow(clippy::doc_markdown)]
#![allow(clippy::used_underscore_items)]

//! Module defining a book.
use super::enums::Language;
use cfg_if::cfg_if;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
cfg_if! { if #[cfg(feature = "ssr")]{

    use sea_orm::entity::prelude::*;

    /// Represents a book as a SeaORM entity.
    ///
    /// # Fields
    ///
    /// * `id`: The unique identifier for the change list item (UUID).
    /// * `creation_time`: The timestamp when the change list item was created.
    /// * `last_update_time`: The timestamp when the change list item was last updated.
    /// * `source_platform`: The platform where the change originated.
    /// * `change_status`: The current status of the change.
    ///
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
    #[sea_orm(table_name = "book")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        /// The unique identifier for the change list item.
        pub id: Uuid,

        /// ISBN
        pub isbn: Option<String>,

        /// The timestamp when the change list item was created.
        pub ingestion_time: NaiveDateTime,

        /// Author id
        pub author_id: Uuid,

        /// Series id
        pub series_id: Uuid,

        /// Link
        pub url: String,

        /// Title
        pub title: String,

        /// Description
        pub description: String,

        /// Language
        pub language: Language
    }


    /// Defines the relations for the `ChangeList` entity.
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

    impl From<Model> for FEFriendlyBook {
        /// Converts a `Model` into a `FEFriendlyBook`.
        ///
        /// # Arguments
        ///
        /// * `val`: The `Model` to convert.
        ///
        /// # Returns
        ///
        /// A new `FEFriendlyBook` with the same data as the `Model`.
        fn from(val: Model) -> Self {
            FEFriendlyBook {
                id: val.id,
                isbn: val.isbn,
                ingestion_time: val.ingestion_time,
                author_id: val.author_id,
                series_id: val.series_id,
                url: val.url,
                title: val.title,
                description: val.description,
                language: val.language,
            }
       }
    }
}}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FEFriendlyBook {
    pub id: Uuid,

    /// ISBN
    pub isbn: Option<String>,

    /// The timestamp when the change list item was created.
    pub ingestion_time: NaiveDateTime,

    /// Author id
    pub author_id: Uuid,

    /// Series id
    pub series_id: Uuid,

    /// Link
    pub url: String,

    /// Title
    pub title: String,

    /// Description
    pub description: String,

    /// Language
    pub language: Language,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    #[test]
    fn test_model_to_changelist_conversion() {
        let id = Uuid::new_v4();
        let isbn = Some("1234567890".to_string());
        let ingestion_time = DateTime::from_timestamp(61, 0).unwrap().naive_utc();
        let author_id = Uuid::new_v4();
        let series_id = Uuid::new_v4();
        let url = "http://example.com".to_string();
        let title = "Example Title".to_string();
        let description = "Example Description".to_string();
        let language = Language::English;

        let model = Model {
            id,
            isbn: isbn.clone(),
            ingestion_time,
            author_id,
            series_id,
            url: url.clone(),
            title: title.clone(),
            description: description.clone(),
            language,
        };

        let fe_friendly_book = FEFriendlyBook::from(model);

        assert_eq!(fe_friendly_book.id, id);
        assert_eq!(fe_friendly_book.isbn, isbn);
        assert_eq!(fe_friendly_book.ingestion_time, ingestion_time);
        assert_eq!(fe_friendly_book.author_id, author_id);
        assert_eq!(fe_friendly_book.series_id, series_id);
        assert_eq!(fe_friendly_book.url, url);
        assert_eq!(fe_friendly_book.title, title);
        assert_eq!(fe_friendly_book.description, description);
        assert_eq!(fe_friendly_book.language, language);
    }
}
