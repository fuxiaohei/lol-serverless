//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "user_token")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub owner_id: i32,
    #[sea_orm(unique)]
    pub value: String,
    pub name: String,
    pub status: String,
    pub usage: String,
    pub created_at: DateTime,
    pub latest_used_at: DateTime,
    pub expired_at: Option<DateTime>,
    pub deleted_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
