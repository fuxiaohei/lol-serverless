use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum WorkerNode {
    Table,
    Id,
    Ip,
    Ipv6,
    Region,
    Hostname,
    IPInfo,
    MachineInfo,
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
                    .table(WorkerNode::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WorkerNode::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(WorkerNode::Ip)
                            .string_len(64)
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(WorkerNode::Ipv6).string_len(128).not_null())
                    .col(ColumnDef::new(WorkerNode::Hostname).string_len(256).not_null())
                    .col(ColumnDef::new(WorkerNode::Region).string_len(32).not_null())
                    .col(ColumnDef::new(WorkerNode::IPInfo).text().not_null())
                    .col(ColumnDef::new(WorkerNode::MachineInfo).text().not_null())
                    .col(ColumnDef::new(WorkerNode::Status).string_len(12).not_null())
                    .col(
                        ColumnDef::new(WorkerNode::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WorkerNode::UpdatedAt)
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
                    .name("idx-workernode-ip")
                    .table(WorkerNode::Table)
                    .col(WorkerNode::Ip)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-workernode-ipv6")
                    .table(WorkerNode::Table)
                    .col(WorkerNode::Ipv6)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-workernode-region")
                    .table(WorkerNode::Table)
                    .col(WorkerNode::Region)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-workernode-status")
                    .table(WorkerNode::Table)
                    .col(WorkerNode::Status)
                    .to_owned(),
            )
            .await?;
        debug!("Migration: m10_workernode_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}