use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum UserInfo {
    Table,
    Id,
    Uuid,
    Name,
    NickName,
    Email,
    Avatar,
    Status,
    Role,
    Password, // if user is created by Oauth-provider(clerk,others). the password is bcrypt(uuid+salt+origin-user-id)
    PasswordSalt, // random string
    OauthProvider,
    OauthUserId,
    OauthEmailId,
    LastLoginAt,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

async fn create_user_info_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(UserInfo::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(UserInfo::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(UserInfo::Uuid)
                        .string_len(128)
                        .unique_key()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(UserInfo::Password)
                        .string_len(128)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(UserInfo::PasswordSalt)
                        .string_len(64)
                        .not_null(),
                )
                .col(ColumnDef::new(UserInfo::Name).string_len(64).not_null())
                .col(ColumnDef::new(UserInfo::NickName).string_len(64).not_null())
                .col(
                    ColumnDef::new(UserInfo::Email)
                        .string_len(256)
                        .not_null()
                        .unique_key(),
                )
                .col(ColumnDef::new(UserInfo::Avatar).string_len(256).not_null())
                .col(ColumnDef::new(UserInfo::Status).string_len(12).not_null())
                .col(ColumnDef::new(UserInfo::Role).string_len(12).not_null())
                .col(
                    ColumnDef::new(UserInfo::OauthProvider)
                        .string_len(24)
                        .not_null(),
                )
                .col(ColumnDef::new(UserInfo::OauthUserId).string_len(256))
                .col(ColumnDef::new(UserInfo::OauthEmailId).string_len(256))
                .col(
                    ColumnDef::new(UserInfo::CreatedAt)
                        .timestamp()
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .not_null(),
                )
                .col(
                    ColumnDef::new(UserInfo::UpdatedAt)
                        .timestamp()
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .not_null(),
                )
                .col(
                    ColumnDef::new(UserInfo::LastLoginAt)
                        .timestamp()
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .not_null(),
                )
                .col(ColumnDef::new(UserInfo::DeletedAt).timestamp().null())
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-user-info-oauth")
                .table(UserInfo::Table)
                .col(UserInfo::OauthProvider)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-user-info-status")
                .table(UserInfo::Table)
                .col(UserInfo::Status)
                .to_owned(),
        )
        .await?;
    Ok(())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_user_info_table(manager).await?;
        debug!("Migration: m02user_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
