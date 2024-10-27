use crate::{models::user_token, DB};
use anyhow::{anyhow, Result};
use sea_orm::{prelude::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use std::ops::Add;
use tracing::debug;

/// TokenUsage is the usage of the token
#[derive(strum::Display, PartialEq, strum::EnumString, Clone)]
#[strum(serialize_all = "lowercase")]
pub enum Usage {
    Session, // web page session token
    Cmdline, // land-cli token
    Worker,  // land-worker token
}

/// TokenStatus is the status of the token
#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Active,
    Deleted,
}

/// create_session creates a new session token
pub async fn create_session(owner_id: i32, expire: i64) -> Result<user_token::Model> {
    let now_time = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
    let name = format!("session-{}-{}", owner_id, now_time);
    create(owner_id, &name, expire, Usage::Session).await
}

/// create creates a new token
pub async fn create(
    owner_id: i32,
    name: &str,
    expire: i64,
    usage: Usage,
) -> Result<user_token::Model> {
    let now = chrono::Utc::now().naive_utc();
    let expired_at = now.add(chrono::TimeDelta::try_seconds(expire).unwrap());
    let value: String = land_helpers::crypt::rand_string(40);
    let token_model = user_token::Model {
        id: 0,
        owner_id,
        value,
        name: name.to_string(),
        status: Status::Active.to_string(),
        created_at: now,
        latest_used_at: now,
        expired_at: Some(expired_at),
        deleted_at: None,
        usage: usage.to_string(),
    };
    let mut token_active_model: user_token::ActiveModel = token_model.into();
    token_active_model.id = Default::default();
    let token_model = token_active_model.insert(DB.get().unwrap()).await?;
    Ok(token_model)
}

/// get_by_value gets an active token by value
pub async fn get_by_value(value: &str, usage: Option<Usage>) -> Result<Option<user_token::Model>> {
    let db = DB.get().unwrap();
    let mut select = user_token::Entity::find()
        .filter(user_token::Column::Value.eq(value))
        .filter(user_token::Column::Status.eq(Status::Active.to_string()));
    if let Some(u) = usage {
        select = select.filter(user_token::Column::Usage.eq(u.to_string()));
    }
    let token = select.one(db).await.map_err(|e| anyhow::anyhow!(e))?;
    if token.is_none() {
        return Ok(None);
    }
    let token = token.unwrap();
    if is_expired(&token) {
        return Err(anyhow!("Token is expired"));
    }
    Ok(Some(token))
}

/// get_by_name gets an active token by name with owner_id
pub async fn get_by_name(
    name: &str,
    owner_id: i32,
    usage: Option<Usage>,
) -> Result<Option<user_token::Model>> {
    let db = DB.get().unwrap();
    let mut select = user_token::Entity::find()
        .filter(user_token::Column::Name.eq(name))
        .filter(user_token::Column::OwnerId.eq(owner_id))
        .filter(user_token::Column::Status.eq(Status::Active.to_string()));
    if let Some(u) = usage {
        select = select.filter(user_token::Column::Usage.eq(u.to_string()));
    }
    let token = select.one(db).await.map_err(|e| anyhow!(e))?;
    Ok(token)
}

/// is_expired checks if the token is expired
pub fn is_expired(model: &user_token::Model) -> bool {
    let now = chrono::Utc::now().naive_utc();
    if let Some(expired_at) = model.expired_at {
        if now > expired_at {
            return true;
        }
    }
    false
}

/// update_last_usage_at updates the last usage time
pub async fn update_last_usage_at(id: i32) -> Result<()> {
    let db = DB.get().unwrap();
    user_token::Entity::update_many()
        .col_expr(
            user_token::Column::LatestUsedAt,
            Expr::value(chrono::Utc::now()),
        )
        .filter(user_token::Column::Id.eq(id))
        .exec(db)
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(())
}

/// set_expired sets a token to expired
pub async fn set_expired(token_id: i32, name: &str) -> Result<()> {
    let name = format!(
        "deleted-{}-{}-{}",
        token_id,
        name,
        chrono::Utc::now().timestamp()
    );
    debug!("set token to expired: {}", name);
    let db = DB.get().unwrap();
    user_token::Entity::update_many()
        .filter(user_token::Column::Id.eq(token_id))
        .col_expr(
            user_token::Column::Status,
            Expr::value(Status::Deleted.to_string()),
        )
        .col_expr(user_token::Column::Name, Expr::value(name))
        .col_expr(
            user_token::Column::DeletedAt,
            Expr::value(chrono::Utc::now()),
        )
        .exec(db)
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(())
}

/// list lists tokens with owner and usage
pub async fn list(owner_id: Option<i32>, usage: Option<Usage>) -> Result<Vec<user_token::Model>> {
    let db = DB.get().unwrap();
    let mut select = user_token::Entity::find()
        .filter(user_token::Column::Status.eq(Status::Active.to_string()));
    if let Some(o) = owner_id {
        select = select.filter(user_token::Column::OwnerId.eq(o));
    }
    if let Some(u) = usage {
        select = select.filter(user_token::Column::Usage.eq(u.to_string()));
    }
    let tokens = select
        .order_by_desc(user_token::Column::LatestUsedAt)
        .all(db)
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(tokens)
}

/// set_usage_at sets a token to expired
pub async fn set_usage_at(id: i32) -> Result<()> {
    let db = DB.get().unwrap();
    let now = chrono::Utc::now().naive_utc();
    user_token::Entity::update_many()
        .col_expr(user_token::Column::LatestUsedAt, Expr::value(now))
        .filter(user_token::Column::Id.eq(id))
        .exec(db)
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(())
}
