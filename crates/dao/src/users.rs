use crate::{models::user_info, tokens, DB};
use anyhow::{anyhow, Result};
use land_utils::crypt::rand_string;
use sea_orm::{prelude::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use short_uuid::ShortUuid;
use std::collections::HashMap;
use tracing::debug;

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum UserStatus {
    Active,
    Disabled,
    Deleted,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum UserRole {
    Normal,
    Admin,
}

/// create creates a new user
pub async fn create(
    name: String,
    nick_name: String,
    mut email: String,
    avatar: String,
    oauth_user_id: String,
    oauth_provider: String,
    password: Option<String>,
    user_role: Option<UserRole>,
) -> Result<user_info::Model> {
    // currently must be clerk-xxx if oauth_provider is not empty
    if !oauth_provider.is_empty() && !oauth_provider.starts_with("clerk@") {
        return Err(anyhow::anyhow!("OAuth provider is not supported"));
    }
    // generate randompassword , and create user
    let password_salt = rand_string(20);
    let password = if let Some(password) = password {
        password
    } else {
        oauth_user_id.clone()
    };
    let full_password = format!("{}{}", password_salt, password);
    let password = bcrypt::hash(full_password, bcrypt::DEFAULT_COST)?;

    let uuid = ShortUuid::generate().to_string();
    // make sure email is not empty
    if email.is_empty() {
        email = format!("{}@email", uuid);
    }

    // role is optional, default is normal
    let role = user_role.unwrap_or(UserRole::Normal).to_string();

    let now = chrono::Utc::now().naive_utc();
    let user_model = user_info::Model {
        id: 0,
        uuid,
        email,
        name,
        password,
        password_salt,
        avatar,
        nick_name,
        status: UserStatus::Active.to_string(),
        role,
        created_at: now,
        last_login_at: now,
        updated_at: now,
        deleted_at: None,
        oauth_user_id: Some(oauth_user_id),
        oauth_email_id: None,
        oauth_provider,
    };
    let mut user_active_model: user_info::ActiveModel = user_model.into();
    user_active_model.id = Default::default();
    let user_model = user_active_model.insert(DB.get().unwrap()).await?;
    Ok(user_model)
}

/// get_by_id finds a user by id
pub async fn get_by_id(id: i32, status: Option<UserStatus>) -> Result<Option<user_info::Model>> {
    let db = DB.get().unwrap();
    let mut select = user_info::Entity::find_by_id(id);
    if let Some(s) = status {
        select = select.filter(user_info::Column::Status.eq(s.to_string()));
    }
    let user = select.one(db).await.map_err(|e| anyhow!(e))?;
    Ok(user)
}

/// verify_password verifies the password
pub fn verify_password(user: &user_info::Model, pwd: &str) -> bool {
    let full_password = format!("{}{}", user.password_salt, pwd);
    bcrypt::verify(full_password, &user.password).unwrap_or(false)
}

/// get_by_email finds a user by email
pub async fn get_by_email(
    email: &str,
    status: Option<UserStatus>,
) -> Result<Option<user_info::Model>> {
    let db = DB.get().unwrap();
    let mut select = user_info::Entity::find();
    if let Some(s) = status {
        select = select.filter(user_info::Column::Status.eq(s.to_string()));
    }
    let user = select
        .filter(user_info::Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(user)
}

/// update_login_at updates the last login time
async fn update_login_at(user_id: i32) -> Result<()> {
    let db = DB.get().unwrap();
    user_info::Entity::update_many()
        .col_expr(
            user_info::Column::LastLoginAt,
            Expr::value(chrono::Utc::now()),
        )
        .filter(user_info::Column::Id.eq(user_id))
        .exec(db)
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(())
}

/// verify_session verifies the session token and returns the user
pub async fn verify_session(session: &str) -> Result<user_info::Model> {
    let token = tokens::get_by_value(session, Some(tokens::Usage::Session)).await?;
    if token.is_none() {
        return Err(anyhow!("Session token not found"));
    }
    let token = token.unwrap();
    let user = get_by_id(token.owner_id, None).await?;
    if user.is_none() {
        return Err(anyhow!("User not found"));
    }
    let user = user.unwrap();
    if user.status == UserStatus::Disabled.to_string() {
        return Err(anyhow!("User is disabled"));
    }
    let now = chrono::Utc::now().naive_utc();
    let diff = now - user.last_login_at;
    // if last login time is more than 60 seconds, update last login time
    if diff.num_seconds() > 60 {
        update_login_at(user.id).await?;
        tokens::update_last_usage_at(token.id).await?;
    }
    debug!(
        "session verified user: {:?}, last_login_at: {:?}",
        user.name, user.last_login_at
    );
    Ok(user)
}

/// list_by_ids returns a map of users by ids
pub async fn list_by_ids(ids: Vec<i32>) -> Result<HashMap<i32, user_info::Model>> {
    let db = DB.get().unwrap();
    let users = user_info::Entity::find()
        .filter(user_info::Column::Id.is_in(ids))
        .all(db)
        .await
        .map_err(|e| anyhow!(e))?;
    let mut map = HashMap::new();
    for user in users {
        map.insert(user.id, user);
    }
    Ok(map)
}
