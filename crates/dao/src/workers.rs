use crate::{models::worker_node, DB};
use anyhow::{anyhow, Result};
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, Order, QueryFilter,
    QueryOrder,
};

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Online,
    Offline,
}

/// find_all returns all worker nodes
pub async fn find_all(status: Option<Status>) -> Result<Vec<worker_node::Model>> {
    let db = DB.get().unwrap();
    let mut select = worker_node::Entity::find();
    if let Some(status) = status {
        select = select.filter(worker_node::Column::Status.eq(status.to_string()));
    }
    let nodes = select
        .order_by(worker_node::Column::UpdatedAt, Order::Desc)
        .order_by(worker_node::Column::Ip, Order::Asc)
        .all(db)
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(nodes)
}

/// set_offline sets worker node offline
pub async fn set_offline(ip: &str) -> Result<()> {
    let db = DB.get().unwrap();
    let now = chrono::Utc::now().naive_utc();
    worker_node::Entity::update_many()
        .col_expr(
            worker_node::Column::Status,
            Expr::value(Status::Offline.to_string()),
        )
        .col_expr(worker_node::Column::UpdatedAt, Expr::value(now))
        .filter(worker_node::Column::Ip.eq(ip))
        .exec(db)
        .await?;
    Ok(())
}

/// set_onlines sets worker nodes online
pub async fn set_onlines(ips: Vec<String>) -> Result<()> {
    let db = DB.get().unwrap();
    let now = chrono::Utc::now().naive_utc();
    worker_node::Entity::update_many()
        .col_expr(
            worker_node::Column::Status,
            Expr::value(Status::Online.to_string()),
        )
        .col_expr(worker_node::Column::UpdatedAt, Expr::value(now))
        .filter(worker_node::Column::Ip.is_in(ips))
        .exec(db)
        .await?;
    Ok(())
}
/// create creates a new worker node
pub async fn create(
    ip: &str,
    ipv6: &str,
    hostname: &str,
    region: &str,
    ip_info: &str,
) -> Result<worker_node::Model> {
    let now = chrono::Utc::now().naive_utc();
    let model = worker_node::Model {
        id: Default::default(),
        ip: ip.to_string(),
        ipv6: ipv6.to_string(),
        hostname: hostname.to_string(),
        region: region.to_string(),
        ip_info: ip_info.to_string(),
        machine_info: "".to_string(),
        status: Status::Online.to_string(),
        created_at: now,
        updated_at: now,
    };
    let mut active_model: worker_node::ActiveModel = model.into();
    active_model.id = ActiveValue::default();
    let db = DB.get().unwrap();
    let project = active_model.insert(db).await?;
    Ok(project)
}
