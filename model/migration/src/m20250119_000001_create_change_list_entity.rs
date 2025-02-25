use entities_lib::entities::enums::{ChangeStatus, SourcePlatform};
use extension::postgres::Type;
use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::{date_time, enumeration, pk_uuid};
use sea_orm_migration::sea_orm::Iterable;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        let change_status_create_stmt = Type::create()
            .values(ChangeStatus::iter())
            .as_enum(ChangeList::ChangeStatus)
            .to_owned();
        manager.create_type(change_status_create_stmt).await?;

        let source_platform_create_stmt = Type::create()
            .values(SourcePlatform::iter())
            .as_enum(ChangeList::SourcePlatform)
            .to_owned();
        manager.create_type(source_platform_create_stmt).await?;

        manager
            .create_table(
                Table::create()
                    .table(ChangeList::Table)
                    .if_not_exists()
                    .col(pk_uuid(ChangeList::Id))
                    .col(date_time(ChangeList::CreationTime))
                    .col(date_time(ChangeList::LastUpdateTime))
                    .col(enumeration(
                        ChangeList::SourcePlatform,
                        ChangeList::SourcePlatform,
                        SourcePlatform::iter(),
                    ))
                    .col(enumeration(
                        ChangeList::ChangeStatus,
                        ChangeList::ChangeStatus,
                        ChangeStatus::iter(),
                    ))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(
                Table::drop()
                    .if_exists()
                    .table(ChangeList::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .if_exists()
                    .name(ChangeList::ChangeStatus)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .if_exists()
                    .name(ChangeList::SourcePlatform)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ChangeList {
    Table,
    Id,
    CreationTime,
    LastUpdateTime,
    SourcePlatform,
    ChangeStatus,
}
