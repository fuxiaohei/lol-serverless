use crate::{models::environment_variables, now_time, DB};
use anyhow::Result;
use land_common::{base64decode, base64encode, rand_string};
use sea_orm::{
    prelude::Expr, ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};
use serde::Deserialize;
use simple_crypt::{decrypt, encrypt};
use std::collections::HashMap;

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
pub async fn get(project_id: i32) -> Result<Option<environment_variables::Model>> {
    let db = DB.get().unwrap();
    let select = environment_variables::Entity::find()
        .filter(environment_variables::Column::ProjectId.eq(project_id))
        .filter(environment_variables::Column::Status.eq(Status::Active.to_string()))
        .order_by_desc(environment_variables::Column::Id);
    let env = select.one(db).await?;
    Ok(env)
}

fn encode_envs(q: EnvsQuery) -> Result<(String, String)> {
    let secret = rand_string(12);
    let q_map = q.into_map();
    let q_value = serde_json::to_vec(&q_map)?;
    let encrypt_data = encrypt(&q_value, secret.as_bytes())?;
    let encrypt_string = base64encode(encrypt_data);
    Ok((secret, encrypt_string))
}

fn encode_envs_map(secret: String, q_map: HashMap<String, String>) -> Result<String> {
    let q_value = serde_json::to_vec(&q_map)?;
    let encrypt_data = encrypt(&q_value, secret.as_bytes())?;
    let encrypt_string = base64encode(encrypt_data);
    Ok(encrypt_string)
}

fn decode_env(secret: &str, encrypt_string: &str) -> Result<HashMap<String, String>> {
    let encrypt_data = base64decode(encrypt_string)?;
    let decrypt_data = decrypt(&encrypt_data, secret.as_bytes())?;
    let q_map = serde_json::from_slice(&decrypt_data)?;
    Ok(q_map)
}

/// update environment variable
pub async fn update(
    m: environment_variables::Model,
    q: EnvsQuery,
) -> Result<environment_variables::Model> {
    let old_map = decode_env(&m.secret_key, &m.content)?;
    let new_map = q.merge_map(old_map);
    let encrypt_string = encode_envs_map(m.secret_key.clone(), new_map)?;
    let task_id = rand_string(12);
    let db = DB.get().unwrap();
    environment_variables::Entity::update_many()
        .col_expr(
            environment_variables::Column::Content,
            Expr::value(encrypt_string),
        )
        .col_expr(environment_variables::Column::TaskId, Expr::value(task_id))
        .col_expr(
            environment_variables::Column::UpdatedAt,
            Expr::value(now_time()),
        )
        .filter(environment_variables::Column::Id.eq(m.id))
        .exec(db)
        .await?;
    Ok(m)
}

/// create environment variable
pub async fn create(
    owner_id: i32,
    project_id: i32,
    q: EnvsQuery,
) -> Result<environment_variables::Model> {
    let (secret, encrypt_string) = encode_envs(q)?;
    let task_id = rand_string(12);
    let now = now_time();
    let env = environment_variables::Model {
        id: 0,
        owner_id,
        project_id,
        task_id,
        secret_key: secret,
        content: encrypt_string,
        status: Status::Active.to_string(),
        created_at: now,
        updated_at: now,
    };
    let mut env_active_model: environment_variables::ActiveModel = env.into();
    env_active_model.id = ActiveValue::default();
    let db = DB.get().unwrap();
    let env = env_active_model.insert(db).await?;
    Ok(env)
}

/// get_keys get keys of environment variable
pub async fn get_keys(m: environment_variables::Model) -> Result<Vec<String>> {
    let old_map = decode_env(&m.secret_key, &m.content)?;
    let mut keys: Vec<String> = old_map.keys().cloned().collect();
    // sort keys
    keys.sort();
    Ok(keys)
}
