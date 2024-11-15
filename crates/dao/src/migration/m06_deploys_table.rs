use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum Deploys {
    Table,
    Id,
    OwnerId,
    OwnerUuid,
    ProjectId,
    ProjectUuid,
    TaskId,
    Domain,
    Spec,
    DeployType,
    DeployStatus,
    DeployMessage,
    Status,
    Rips,
    Description,
    SuccessCount,
    FailedCount,
    TotalCount,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

async fn create_deploys_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Deploys::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Deploys::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(Deploys::OwnerId).integer().not_null())
                .col(ColumnDef::new(Deploys::OwnerUuid).string_len(64).not_null())
                .col(ColumnDef::new(Deploys::ProjectId).integer().not_null())
                .col(
                    ColumnDef::new(Deploys::ProjectUuid)
                        .string_len(64)
                        .not_null(),
                )
                .col(ColumnDef::new(Deploys::TaskId).string_len(64).not_null())
                .col(ColumnDef::new(Deploys::Domain).string_len(128).not_null())
                .col(ColumnDef::new(Deploys::Spec).json().not_null())
                .col(
                    ColumnDef::new(Deploys::DeployType)
                        .string_len(12)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Deploys::DeployStatus)
                        .string_len(12)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Deploys::DeployMessage)
                        .string_len(256)
                        .not_null(),
                )
                .col(ColumnDef::new(Deploys::Status).string_len(12).not_null())
                .col(ColumnDef::new(Deploys::Rips).text().not_null())
                .col(ColumnDef::new(Deploys::Description).text().not_null())
                .col(ColumnDef::new(Deploys::SuccessCount).integer().not_null())
                .col(ColumnDef::new(Deploys::FailedCount).integer().not_null())
                .col(ColumnDef::new(Deploys::TotalCount).integer().not_null())
                .col(
                    ColumnDef::new(Deploys::CreatedAt)
                        .timestamp()
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Deploys::UpdatedAt)
                        .timestamp()
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .not_null(),
                )
                .col(ColumnDef::new(Deploys::DeletedAt).timestamp().null())
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-domain")
                .table(Deploys::Table)
                .col(Deploys::Domain)
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-ownerid")
                .table(Deploys::Table)
                .col(Deploys::OwnerId)
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-owner-uuid")
                .table(Deploys::Table)
                .col(Deploys::OwnerUuid)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-project-id")
                .table(Deploys::Table)
                .col(Deploys::ProjectId)
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-project-uuid")
                .table(Deploys::Table)
                .col(Deploys::ProjectUuid)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-taskid")
                .table(Deploys::Table)
                .col(Deploys::TaskId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-status")
                .table(Deploys::Table)
                .col(Deploys::Status)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-deploy-status")
                .table(Deploys::Table)
                .col(Deploys::DeployStatus)
                .to_owned(),
        )
        .await?;

    Ok(())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_deploys_table(manager).await?;
        debug!("Migration: m06_deploys_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
