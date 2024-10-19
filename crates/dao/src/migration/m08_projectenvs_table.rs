use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum ProjectEnvs {
    Table,
    Id,
    OwnerId,
    ProjectId,
    TaskId, // use to create deploy task
    Content,
    SecretKey,
    Scope, // scope means the envs used for one project or on account or other
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
                    .table(ProjectEnvs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectEnvs::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ProjectEnvs::OwnerId).integer().not_null())
                    .col(ColumnDef::new(ProjectEnvs::ProjectId).integer().not_null())
                    .col(
                        ColumnDef::new(ProjectEnvs::TaskId)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectEnvs::SecretKey)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProjectEnvs::Scope).string_len(12).not_null())
                    .col(ColumnDef::new(ProjectEnvs::Content).text().not_null())
                    .col(
                        ColumnDef::new(ProjectEnvs::Status)
                            .string_len(12)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectEnvs::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectEnvs::UpdatedAt)
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
                    .table(ProjectEnvs::Table)
                    .col(ProjectEnvs::OwnerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-envs-project-id")
                    .table(ProjectEnvs::Table)
                    .col(ProjectEnvs::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-envs-taskid")
                    .table(ProjectEnvs::Table)
                    .col(ProjectEnvs::TaskId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-envs-status")
                    .table(ProjectEnvs::Table)
                    .col(ProjectEnvs::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-envs-scope")
                    .table(ProjectEnvs::Table)
                    .col(ProjectEnvs::Scope)
                    .to_owned(),
            )
            .await?;
        debug!("Migration: m08_projectenvs_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
