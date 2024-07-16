//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "user_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub uuid: String,
    pub password: String,
    pub password_salt: String,
    pub name: String,
    pub nick_name: String,
    #[sea_orm(unique)]
    pub email: String,
    pub avatar: String,
    pub status: String,
    pub role: String,
    pub oauth_provider: String,
    pub oauth_user_id: Option<String>,
    pub oauth_email_id: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub last_login_at: DateTime,
    pub deleted_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
