use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum WasmArtifacts {
    Table,
    Id,
    OwnerId,
    ProjectId,
    DeployId,
    TaskId,
    Path,
    FileSize,
    FileHash,
    FileTarget,
    Status,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WasmArtifacts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WasmArtifacts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(WasmArtifacts::OwnerId).integer().not_null())
                    .col(
                        ColumnDef::new(WasmArtifacts::ProjectId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(WasmArtifacts::DeployId).integer().not_null())
                    .col(
                        ColumnDef::new(WasmArtifacts::TaskId)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WasmArtifacts::Path)
                            .string_len(256)
                            .not_null(),
                    )
                    .col(ColumnDef::new(WasmArtifacts::FileSize).integer().not_null())
                    .col(
                        ColumnDef::new(WasmArtifacts::FileHash)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WasmArtifacts::FileTarget)
                            .string_len(256)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WasmArtifacts::Status)
                            .string_len(12)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WasmArtifacts::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WasmArtifacts::UpdatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(ColumnDef::new(WasmArtifacts::DeletedAt).timestamp().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-wasm-artifacts-ownerid")
                    .table(WasmArtifacts::Table)
                    .col(WasmArtifacts::OwnerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-wasm-artifacts-project-id")
                    .table(WasmArtifacts::Table)
                    .col(WasmArtifacts::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-wasm-artifacts-deploy-id")
                    .table(WasmArtifacts::Table)
                    .col(WasmArtifacts::DeployId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-wasm-artifacts-task-id")
                    .table(WasmArtifacts::Table)
                    .col(WasmArtifacts::TaskId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-wasm-artifacts-status")
                    .table(WasmArtifacts::Table)
                    .col(WasmArtifacts::Status)
                    .to_owned(),
            )
            .await?;

        debug!("Migration: m09_wasm_artifacts_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
