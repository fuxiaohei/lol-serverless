//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.0

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "deploys")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub owner_id: i32,
    pub owner_uuid: String,
    pub project_id: i32,
    pub project_uuid: String,
    pub task_id: String,
    pub domain: String,
    pub spec: Json,
    pub deploy_type: String,
    pub deploy_status: String,
    pub deploy_message: String,
    pub status: String,
    #[sea_orm(column_type = "Text")]
    pub rips: String,
    pub success_count: i32,
    pub failed_count: i32,
    pub total_count: i32,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub deleted_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
