//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.14

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "deploy_task")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub ip: String,
    pub worker_id: i32,
    pub project_id: i32,
    pub deployment_id: i32,
    pub task_id: String,
    pub status: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub message: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
