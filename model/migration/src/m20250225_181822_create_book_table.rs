#![allow(clippy::enum_variant_names)]
use entities_lib::entities::enums::Language;
use extension::postgres::Type;
use sea_orm_migration::schema::{date_time, pk_uuid, text_null};
use sea_orm_migration::sea_orm::Iterable;
use sea_orm_migration::{
    prelude::*,
    schema::{text, uuid},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        let change_status_create_stmt = Type::create()
            .values(Language::iter())
            .as_enum(Book::BookLanguage)
            .to_owned();

        manager.create_type(change_status_create_stmt).await?;
        manager
            .create_table(
                Table::create()
                    .table(Book::Table)
                    .if_not_exists()
                    .col(pk_uuid(Book::Id))
                    .col(text_null(Book::Isbn))
                    .col(date_time(Book::IngestionTime))
                    .col(uuid(Book::AuthorID))
                    .col(uuid(Book::SeriesID))
                    .col(text(Book::Url))
                    .col(text(Book::Title))
                    .col(text(Book::Description))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().if_exists().table(Book::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().if_exists().name(Book::BookLanguage).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Book {
    Table,
    Id,
    Isbn,
    IngestionTime,
    AuthorID,
    SeriesID,
    Url,
    Title,
    Description,
    BookLanguage,
}
