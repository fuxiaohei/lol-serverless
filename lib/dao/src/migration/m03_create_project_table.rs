use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum Project {
    Table,
    Id,
    Uuid,
    OwnerId,
    Name,
    Language,
    DevDomain,
    ProdDomain,
    Description,
    DeployStatus,
    Status,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Metadata,
}

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

async fn create_project_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Project::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Project::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(Project::OwnerId).integer().not_null())
                .col(ColumnDef::new(Project::Name).string_len(128).not_null())
                .col(ColumnDef::new(Project::Language).string_len(24).not_null())
                .col(ColumnDef::new(Project::Status).string_len(24).not_null())
                .col(
                    ColumnDef::new(Project::DeployStatus)
                        .string_len(24)
                        .not_null(),
                )
                .col(ColumnDef::new(Project::Uuid).string_len(64).not_null())
                .col(
                    ColumnDef::new(Project::Description)
                        .string_len(256)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Project::DevDomain)
                        .string_len(256)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Project::ProdDomain)
                        .string_len(256)
                        .not_null(),
                )
                .col(ColumnDef::new(Project::CreatedBy).string_len(24).not_null())
                .col(
                    ColumnDef::new(Project::CreatedAt)
                        .timestamp()
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Project::UpdatedAt)
                        .timestamp()
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .not_null(),
                )
                .col(ColumnDef::new(Project::DeletedAt).timestamp().null())
                .col(ColumnDef::new(Project::Metadata).text().null())
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-project-info-ownerid-name")
                .table(Project::Table)
                .col(Project::Name)
                .col(Project::OwnerId)
                .unique()
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-project-info-status")
                .table(Project::Table)
                .col(Project::Status)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-project-info-deploystatus")
                .table(Project::Table)
                .col(Project::DeployStatus)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-project-info-created-by")
                .table(Project::Table)
                .col(Project::CreatedBy)
                .to_owned(),
        )
        .await?;
    Ok(())
}

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
        create_project_table(manager).await?;
        create_playground_table(manager).await?;
        debug!("Migration: m03_create_project_table has been applied");
        Ok(())
    }
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
