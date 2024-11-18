use crate::{
    models::{deploy_state, deploys},
    DB,
};
use anyhow::Result;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel,
    ItemsAndPagesNumber, PaginatorTrait, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    WaitDeploy,
    Compiling, // if compilation is long time, we need mark it as compiling
    Uploading,
    Deploying,
    Success,
    Failed,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum DeploymentStatus {
    Active,
    Disabled, // if a deployment is disabled, it will not be deployed
    Deleted,  // if a deployment is deleted, it will not be shown
    Outdated, // if a deployment is outdated, it will be deleted
}

#[derive(strum::Display, PartialEq, strum::EnumString, Clone)]
#[strum(serialize_all = "lowercase")]
pub enum DeployType {
    Production,  // production deployment
    Development, // development deployment
    Disabled,    // disabled deployment
    Envs,        // envs deployment
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum StateType {
    WasmDeploy, // wasm deployment
    Envs,       // envs deployment
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Spec {
    cpu_time_limit: Option<i32>,
    memory_limit: Option<i32>,
    wall_time_limit: Option<i32>,
    fetch_limit: Option<i32>,
}

impl Default for Spec {
    fn default() -> Self {
        Self {
            cpu_time_limit: Some(100), // 100ms
            memory_limit: Some(128),   // 128MB
            wall_time_limit: Some(30), // 30 seconds
            fetch_limit: Some(5),      // send 5 requests
        }
    }
}

/// create a deployment
pub async fn create(
    owner_id: i32,
    owner_uuid: String,
    project_id: i32,
    project_uuid: String,
    domain: String,
    deploy_type: DeployType,
    description: String,
) -> Result<deploys::Model> {
    let spec = Spec::default();
    let now = chrono::Utc::now().naive_utc();
    let model = deploys::Model {
        id: 0,
        owner_id,
        owner_uuid,
        project_id,
        project_uuid,
        task_id: Uuid::new_v4().to_string(),
        domain,
        spec: serde_json::to_value(&spec)?,
        deploy_type: deploy_type.to_string(),
        deploy_status: Status::WaitDeploy.to_string(),
        deploy_message: "Waiting to deploy".to_string(),
        status: DeploymentStatus::Active.to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        rips: String::new(),
        description,
        success_count: 0,
        failed_count: 0,
        total_count: 0,
    };
    let mut active_model = model.into_active_model();
    active_model.id = Default::default();
    let db = DB.get().unwrap();
    let model = active_model.insert(db).await?;
    Ok(model)
}

/// list_by_deploy_status returns a list of deployments by deploy status
pub async fn list_by_deploy_status(status: Status) -> Result<Vec<deploys::Model>> {
    let db = DB.get().unwrap();
    let models = deploys::Entity::find()
        .filter(deploys::Column::DeployStatus.contains(status.to_string()))
        .all(db)
        .await?;
    Ok(models)
}

/// set_deploy_status sets the status of a deployment
pub async fn set_deploy_status(deploy_id: i32, status: Status, message: &str) -> Result<()> {
    let db = DB.get().unwrap();
    let now = chrono::Utc::now();
    deploys::Entity::update_many()
        .col_expr(
            deploys::Column::DeployStatus,
            Expr::value(status.to_string()),
        )
        .col_expr(deploys::Column::DeployMessage, Expr::value(message))
        .col_expr(deploys::Column::UpdatedAt, Expr::value(now))
        .filter(deploys::Column::Id.eq(deploy_id))
        .exec(db)
        .await?;
    Ok(())
}

/// set_rips sets the rips of a deployment
pub async fn set_rips(id: i32, rips: String, total_count: i32) -> Result<()> {
    let db = DB.get().unwrap();
    let now = chrono::Utc::now();
    deploys::Entity::update_many()
        .col_expr(deploys::Column::Rips, Expr::value(rips))
        .col_expr(deploys::Column::TotalCount, Expr::value(total_count))
        .col_expr(deploys::Column::UpdatedAt, Expr::value(now))
        .filter(deploys::Column::Id.eq(id))
        .exec(db)
        .await?;
    Ok(())
}

/// success_ids returns a list of success deployment ids
pub async fn success_ids() -> Result<Vec<i32>> {
    let db = DB.get().unwrap();
    let models = deploy_state::Entity::find()
        .filter(deploy_state::Column::StateType.eq(StateType::WasmDeploy.to_string()))
        .order_by_asc(deploy_state::Column::Id)
        .all(db)
        .await?;
    let mut deploy_ids: Vec<i32> = models.iter().map(|model| model.deploy_id).collect();
    deploy_ids.sort();
    Ok(deploy_ids)
}

/// list_by_ids returns a list of deployments by ids
pub async fn list_by_ids(ids: Vec<i32>) -> Result<Vec<deploys::Model>> {
    let db = DB.get().unwrap();
    let models = deploys::Entity::find()
        .filter(deploys::Column::Id.is_in(ids))
        .all(db)
        .await?;
    Ok(models)
}

/// list_by returns a list of deployments by project
pub async fn list_by(
    project_id: Option<i32>,
    deploy_types: Vec<DeployType>,
    page: u64,
    page_size: u64,
) -> Result<(Vec<deploys::Model>, ItemsAndPagesNumber)> {
    let db = DB.get().unwrap();
    let mut selector = deploys::Entity::find();
    if let Some(project_id) = project_id {
        selector = selector.filter(deploys::Column::ProjectId.eq(project_id));
    }
    if !deploy_types.is_empty() {
        let types: Vec<String> = deploy_types.iter().map(|t| t.to_string()).collect();
        selector = selector.filter(deploys::Column::DeployType.is_in(types));
    }
    let pager = selector
        .order_by_desc(deploys::Column::Id)
        .paginate(db, page_size);
    let models = pager.fetch_page(page - 1).await?;
    let pages = pager.num_items_and_pages().await?;
    Ok((models, pages))
}

/// get_for_status returns a deployment by status
pub async fn get_for_status(id: i32, task_id: String) -> Result<Option<deploys::Model>> {
    let db = DB.get().unwrap();
    let model = deploys::Entity::find()
        .filter(deploys::Column::Id.eq(id))
        .filter(deploys::Column::TaskId.eq(task_id))
        .one(db)
        .await?;
    Ok(model)
}

/// refresh_state updates the state of a deployment
pub async fn refresh_state(
    owner_id: i32,
    project_id: i32,
    deploy_id: i32,
    task_id: String,
    state_type: StateType,
) -> Result<deploy_state::Model> {
    let db = DB.get().unwrap();
    let now = chrono::Utc::now().naive_utc();
    let model = deploy_state::Entity::find()
        .filter(deploy_state::Column::OwnerId.eq(owner_id))
        .filter(deploy_state::Column::ProjectId.eq(project_id))
        .filter(deploy_state::Column::StateType.eq(state_type.to_string()))
        .one(db)
        .await?;
    if let Some(model) = model {
        deploy_state::Entity::update_many()
            .col_expr(deploy_state::Column::DeployId, Expr::value(deploy_id))
            .col_expr(deploy_state::Column::TaskId, Expr::value(task_id))
            .col_expr(deploy_state::Column::UpdatedAt, Expr::value(now))
            .filter(deploy_state::Column::Id.eq(model.id))
            .exec(db)
            .await?;
        Ok(model)
    } else {
        let model = deploy_state::Model {
            id: 0,
            owner_id,
            project_id,
            deploy_id,
            task_id,
            state_type: state_type.to_string(),
            value: "".to_string(),
            created_at: now,
            updated_at: now,
        };
        let mut active_model = model.into_active_model();
        active_model.id = Default::default();
        let model = active_model.insert(db).await?;
        Ok(model)
    }
}

/// drop_state
pub async fn drop_state(
    owner_id: i32,
    project_id: i32,
    deploy_id: i32,
    state_type: StateType,
) -> Result<()> {
    let db = DB.get().unwrap();
    deploy_state::Entity::delete_many()
        .filter(deploy_state::Column::OwnerId.eq(owner_id))
        .filter(deploy_state::Column::ProjectId.eq(project_id))
        .filter(deploy_state::Column::DeployId.eq(deploy_id))
        .filter(deploy_state::Column::StateType.eq(state_type.to_string()))
        .exec(db)
        .await?;
    Ok(())
}
