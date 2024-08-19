use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum EnvironmentVariables {
    Table,
    Id,
    OwnerId,
    ProjectId,
    TaskId, // use to create deploy task
    Content,
    SecretKey,
    Status,
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
                    .table(EnvironmentVariables::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EnvironmentVariables::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(EnvironmentVariables::OwnerId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EnvironmentVariables::ProjectId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EnvironmentVariables::TaskId)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EnvironmentVariables::SecretKey)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EnvironmentVariables::Content)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EnvironmentVariables::Status)
                            .string_len(12)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EnvironmentVariables::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EnvironmentVariables::UpdatedAt)
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
                    .name("idx-envs-ownerid")
                    .table(EnvironmentVariables::Table)
                    .col(EnvironmentVariables::OwnerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-envs-project-id")
                    .table(EnvironmentVariables::Table)
                    .col(EnvironmentVariables::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-envs-taskid")
                    .table(EnvironmentVariables::Table)
                    .col(EnvironmentVariables::TaskId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-envs-status")
                    .table(EnvironmentVariables::Table)
                    .col(EnvironmentVariables::Status)
                    .to_owned(),
            )
            .await?;
        debug!("Migration: m08_create_envs_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
