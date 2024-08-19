use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum Project {
    Table,
    DeployId,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::alter()
            .table(Project::Table)
            .add_column(
                ColumnDef::new(Project::DeployId)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .to_owned();
        manager.alter_table(table).await?;
        debug!("Migration: m09_alter_project_deployid has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
