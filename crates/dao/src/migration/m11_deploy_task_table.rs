use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum DeployTask {
    Table,
    Id,
    OwnerId,
    ProjectId,
    DeployId,
    TaskId,
    TaskType,
    WorkerId,
    WorkerIp,
    TaskContent,
    Status,
    Message,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DeployTask::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DeployTask::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DeployTask::OwnerId).integer().not_null())
                    .col(ColumnDef::new(DeployTask::ProjectId).integer().not_null())
                    .col(ColumnDef::new(DeployTask::DeployId).integer().not_null())
                    .col(ColumnDef::new(DeployTask::TaskId).string_len(64).not_null())
                    .col(
                        ColumnDef::new(DeployTask::TaskType)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(ColumnDef::new(DeployTask::WorkerId).integer().not_null())
                    .col(
                        ColumnDef::new(DeployTask::WorkerIp)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(DeployTask::TaskContent).text().not_null())
                    .col(ColumnDef::new(DeployTask::Status).string_len(12).not_null())
                    .col(
                        ColumnDef::new(DeployTask::Message)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DeployTask::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DeployTask::UpdatedAt)
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
                    .name("idx-deploy-task-ownerid")
                    .table(DeployTask::Table)
                    .col(DeployTask::OwnerId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-deploy-task-project-id")
                    .table(DeployTask::Table)
                    .col(DeployTask::ProjectId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-deploy-task-taskid")
                    .table(DeployTask::Table)
                    .col(DeployTask::TaskId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-deploy-task-worker-id")
                    .table(DeployTask::Table)
                    .col(DeployTask::WorkerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-deploy-task-worker-ip")
                    .table(DeployTask::Table)
                    .col(DeployTask::WorkerIp)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-deploy-task-status")
                    .table(DeployTask::Table)
                    .col(DeployTask::Status)
                    .to_owned(),
            )
            .await?;

        debug!("Migration: m11_deploy_task_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
