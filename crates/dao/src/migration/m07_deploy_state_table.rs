use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum DeployState {
    Table,
    Id,
    OwnerId,
    ProjectId,
    DeployId,
    TaskId,
    StateType,
    Value,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

async fn create_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(DeployState::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(DeployState::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(DeployState::OwnerId).integer().not_null())
                .col(ColumnDef::new(DeployState::ProjectId).integer().not_null())
                .col(ColumnDef::new(DeployState::DeployId).integer().not_null())
                .col(
                    ColumnDef::new(DeployState::TaskId)
                        .string_len(64)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(DeployState::StateType)
                        .string_len(64)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(DeployState::Value)
                        .string_len(256)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(DeployState::CreatedAt)
                        .timestamp()
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .not_null(),
                )
                .col(
                    ColumnDef::new(DeployState::UpdatedAt)
                        .timestamp()
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .not_null(),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deploy-state-ownerid")
                .table(DeployState::Table)
                .col(DeployState::OwnerId)
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deploy-state-projectid")
                .table(DeployState::Table)
                .col(DeployState::ProjectId)
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deploy-state-deployid")
                .table(DeployState::Table)
                .col(DeployState::DeployId)
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deploy-state-taskid")
                .table(DeployState::Table)
                .col(DeployState::TaskId)
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deploy-state-type")
                .table(DeployState::Table)
                .col(DeployState::StateType)
                .to_owned(),
        )
        .await?;

    Ok(())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_table(manager).await?;
        debug!("Migration: m07_deploy_stat_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
