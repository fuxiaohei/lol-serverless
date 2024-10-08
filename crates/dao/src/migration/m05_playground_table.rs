use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum Playground {
    Table,
    Id,
    Uuid,
    OwnerId,
    ProjectId,
    Language,
    Source,
    Version,
    Visiblity,
    Status,
    CreatedAt,
    DeletedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

async fn create_playground_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Playground::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Playground::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(Playground::OwnerId).integer().not_null())
                .col(ColumnDef::new(Playground::ProjectId).integer().not_null())
                .col(ColumnDef::new(Playground::Uuid).string_len(64).not_null())
                .col(
                    ColumnDef::new(Playground::Language)
                        .string_len(24)
                        .not_null(),
                )
                .col(ColumnDef::new(Playground::Source).text().not_null())
                .col(ColumnDef::new(Playground::Status).string_len(12).not_null())
                .col(
                    ColumnDef::new(Playground::Version)
                        .string_len(24)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Playground::Visiblity)
                        .string_len(12)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Playground::CreatedAt)
                        .timestamp()
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .not_null(),
                )
                .col(ColumnDef::new(Playground::DeletedAt).timestamp().null())
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-playground-ownerid")
                .table(Playground::Table)
                .col(Playground::OwnerId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-playground-projectid")
                .table(Playground::Table)
                .col(Playground::ProjectId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-playground-status")
                .table(Playground::Table)
                .col(Playground::Status)
                .to_owned(),
        )
        .await?;
    Ok(())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_playground_table(manager).await?;
        debug!("Migration: m05_playground_table has been applied");
        Ok(())
    }
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
