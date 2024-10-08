use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum Deployment {
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
    SuccessCount,
    FailedCount,
    TotalCount,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

async fn create_deployment_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Deployment::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Deployment::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(Deployment::OwnerId).integer().not_null())
                .col(
                    ColumnDef::new(Deployment::OwnerUuid)
                        .string_len(64)
                        .not_null(),
                )
                .col(ColumnDef::new(Deployment::ProjectId).integer().not_null())
                .col(
                    ColumnDef::new(Deployment::ProjectUuid)
                        .string_len(64)
                        .not_null(),
                )
                .col(ColumnDef::new(Deployment::TaskId).string_len(64).not_null())
                .col(
                    ColumnDef::new(Deployment::Domain)
                        .string_len(128)
                        .not_null(),
                )
                .col(ColumnDef::new(Deployment::Spec).json().not_null())
                .col(
                    ColumnDef::new(Deployment::DeployType)
                        .string_len(12)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Deployment::DeployStatus)
                        .string_len(12)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Deployment::DeployMessage)
                        .string_len(256)
                        .not_null(),
                )
                .col(ColumnDef::new(Deployment::Status).string_len(12).not_null())
                .col(ColumnDef::new(Deployment::Rips).text().not_null())
                .col(
                    ColumnDef::new(Deployment::SuccessCount)
                        .integer()
                        .not_null(),
                )
                .col(ColumnDef::new(Deployment::FailedCount).integer().not_null())
                .col(ColumnDef::new(Deployment::TotalCount).integer().not_null())
                .col(
                    ColumnDef::new(Deployment::CreatedAt)
                        .timestamp()
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Deployment::UpdatedAt)
                        .timestamp()
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .not_null(),
                )
                .col(ColumnDef::new(Deployment::DeletedAt).timestamp().null())
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-domain")
                .table(Deployment::Table)
                .col(Deployment::Domain)
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-ownerid")
                .table(Deployment::Table)
                .col(Deployment::OwnerId)
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-owner-uuid")
                .table(Deployment::Table)
                .col(Deployment::OwnerUuid)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-project-id")
                .table(Deployment::Table)
                .col(Deployment::ProjectId)
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-project-uuid")
                .table(Deployment::Table)
                .col(Deployment::ProjectUuid)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-taskid")
                .table(Deployment::Table)
                .col(Deployment::TaskId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-status")
                .table(Deployment::Table)
                .col(Deployment::Status)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-deploy-status")
                .table(Deployment::Table)
                .col(Deployment::DeployStatus)
                .to_owned(),
        )
        .await?;

    Ok(())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_deployment_table(manager).await?;
        debug!("Migration: m06_deployment_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
