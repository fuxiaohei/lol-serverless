use crate::{models::project_envs, DB};
use anyhow::Result;
use land_utils::crypt::{self, rand_string};
use sea_orm::{
    prelude::Expr, ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Scope {
    Project, // project scope
    Account, // account scope, effect all projects of account
}

#[derive(Deserialize, Debug)]
pub struct EnvsQuery {
    pub key: Vec<String>,
    pub value: Vec<String>,
    pub action: Vec<String>,
}

impl EnvsQuery {
    /// into_map convert EnvsQuery to HashMap<String, String>
    pub fn into_map(self) -> HashMap<String, String> {
        let empty_string = "".to_string();
        let mut map = HashMap::new();
        for (i, k) in self.key.iter().enumerate() {
            if k.is_empty() {
                continue;
            }
            let v = self.value.get(i).unwrap_or(&empty_string);
            map.insert(k.clone(), v.clone());
        }
        map
    }
    pub fn merge_map(self, mut m: HashMap<String, String>) -> HashMap<String, String> {
        let empty_string = "".to_string();
        let mut keys = vec![];
        for (k, _) in m.iter() {
            // if k not in self.keys
            if !self.key.contains(k) {
                keys.push(k.clone());
            }
        }
        for k in keys {
            m.remove(&k);
        }
        for (i, k) in self.key.iter().enumerate() {
            if k.is_empty() {
                continue;
            }
            let action = self.action.get(i).unwrap_or(&empty_string);
            if action == "add" {
                let v = self.value.get(i).unwrap_or(&empty_string);
                m.insert(k.clone(), v.clone());
                continue;
            }
        }
        m
    }
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Active,
    Expired,
}

/// get environment variable by project id
pub async fn get(project_id: i32) -> Result<Option<project_envs::Model>> {
    let db = DB.get().unwrap();
    let select = project_envs::Entity::find()
        .filter(project_envs::Column::ProjectId.eq(project_id))
        .filter(project_envs::Column::Status.eq(Status::Active.to_string()))
        .order_by_desc(project_envs::Column::Id);
    let env = select.one(db).await?;
    Ok(env)
}

/// update environment variable
pub async fn update(m: project_envs::Model, q: EnvsQuery) -> Result<project_envs::Model> {
    let old_map = crypt::decode(&m.secret_key, &m.content)?;
    let new_map = q.merge_map(old_map);
    let encrypt_string = crypt::encode_map_with_secret(new_map, &m.secret_key)?;
    let task_id = rand_string(12);
    let db = DB.get().unwrap();
    let now = chrono::Utc::now();
    project_envs::Entity::update_many()
        .col_expr(project_envs::Column::Content, Expr::value(encrypt_string))
        .col_expr(project_envs::Column::TaskId, Expr::value(task_id))
        .col_expr(project_envs::Column::UpdatedAt, Expr::value(now))
        .filter(project_envs::Column::Id.eq(m.id))
        .exec(db)
        .await?;
    Ok(m)
}

/// create environment variable
pub async fn create(owner_id: i32, project_id: i32, q: EnvsQuery) -> Result<project_envs::Model> {
    let (secret, encrypt_string) = crypt::encode_map(q.into_map())?;
    let task_id = rand_string(12);
    let now = chrono::Utc::now();
    let env = project_envs::Model {
        id: 0,
        owner_id,
        project_id,
        task_id,
        secret_key: secret,
        content: encrypt_string,
        scope: Scope::Project.to_string(),
        status: Status::Active.to_string(),
        created_at: now,
        updated_at: now,
    };
    let mut env_active_model: project_envs::ActiveModel = env.into();
    env_active_model.id = ActiveValue::default();
    let db = DB.get().unwrap();
    let env = env_active_model.insert(db).await?;
    Ok(env)
}

/// get_keys get keys of environment variable
pub async fn get_keys(m: project_envs::Model) -> Result<Vec<String>> {
    let old_map = crypt::decode(&m.secret_key, &m.content)?;
    let mut keys: Vec<String> = old_map.keys().cloned().collect();
    // sort keys
    keys.sort();
    Ok(keys)
}
