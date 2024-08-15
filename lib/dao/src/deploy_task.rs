use crate::{
    models::{deploy_task, deployment},
    now_time, DB,
};
use anyhow::Result;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder,
};

#[derive(strum::Display,strum::EnumString,PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum TaskType {
    DeployWasmToWorker, // deploy wasm to worker
    DisableWasm, // disable wasm
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Doing,
    Success,
    Failed,
}

/// create deploy task
pub async fn create(
    dp: &deployment::Model,
    task_type: TaskType,
    task_content: &str,
    worker_id: i32,
    worker_ip: &str,
) -> Result<deploy_task::Model> {
    let now = now_time();
    let model = deploy_task::Model {
        id: 0,
        owner_id: dp.owner_id,
        project_id: dp.project_id,
        deploy_id: dp.id,
        task_id: dp.task_id.clone(),
        task_type: task_type.to_string(),
        task_content: task_content.to_string(),
        worker_id,
        worker_ip: worker_ip.to_string(),
        status: Status::Doing.to_string(),
        created_at: now,
        updated_at: now,
        message: "".to_string(),
    };
    let mut active_model = model.into_active_model();
    active_model.id = Default::default();
    let db = DB.get().unwrap();
    let model = active_model.insert(db).await?;
    Ok(model)
}

/// list deploy task
pub async fn list(
    ip: Option<String>,
    status: Option<Status>,
    task_id: Option<String>,
) -> Result<Vec<deploy_task::Model>> {
    let db = DB.get().unwrap();
    let mut select = deploy_task::Entity::find();
    if let Some(ip) = ip {
        select = select.filter(deploy_task::Column::WorkerIp.eq(ip));
    }
    if let Some(status) = status {
        select = select.filter(deploy_task::Column::Status.eq(status.to_string()));
    }
    if let Some(task_id) = task_id {
        select = select.filter(deploy_task::Column::TaskId.eq(task_id));
    }
    let models = select.order_by_asc(deploy_task::Column::Id).all(db).await?;
    Ok(models)
}

/// set_success set task success
pub async fn set_success(ip: String, task_id: String) -> Result<()> {
    let db = DB.get().unwrap();
    deploy_task::Entity::update_many()
        .col_expr(
            deploy_task::Column::Status,
            Expr::value(Status::Success.to_string()),
        )
        .col_expr(deploy_task::Column::UpdatedAt, Expr::value(now_time()))
        .filter(deploy_task::Column::WorkerIp.eq(ip))
        .filter(deploy_task::Column::TaskId.eq(task_id))
        .exec(db)
        .await?;
    Ok(())
}

/// set_failed set task failed
pub async fn set_failed(ip: String, task_id: String, message: String) -> Result<()> {
    let db = DB.get().unwrap();
    deploy_task::Entity::update_many()
        .col_expr(
            deploy_task::Column::Status,
            Expr::value(Status::Failed.to_string()),
        )
        .col_expr(deploy_task::Column::UpdatedAt, Expr::value(now_time()))
        .col_expr(deploy_task::Column::Message, Expr::value(message))
        .filter(deploy_task::Column::WorkerIp.eq(ip))
        .filter(deploy_task::Column::TaskId.eq(task_id))
        .exec(db)
        .await?;
    Ok(())
}
