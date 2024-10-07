use land_dao::{
    models::{user_info, user_token},
    users::UserRole,
};
use serde::{Deserialize, Serialize};

/// AuthUser is the user info for auth
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AuthUser {
    pub id: i32,
    pub uuid: String,
    pub username: String,
    pub name: String,
    pub email: String,
    pub avatar_url: String,
    pub social_name: Option<String>,
    pub social_provider: Option<String>,
    pub social_link: Option<String>,
    pub is_admin: bool,
    pub last_login_at: i64,
    pub created_at: i64,
    pub status: String,
    pub projects_count: Option<i64>,
}

impl AuthUser {
    /// new creates a new auth user from user_info::Model
    pub fn new(user: &user_info::Model) -> Self {
        let mut u = AuthUser {
            id: user.id,
            uuid: user.uuid.clone(),
            username: user.name.clone(),
            name: user.nick_name.clone(),
            email: user.email.clone(),
            avatar_url: user.avatar.clone(),
            social_name: None,
            social_provider: None,
            social_link: None,
            is_admin: user.role == UserRole::Admin.to_string(),
            last_login_at: user.last_login_at.timestamp(),
            created_at: user.created_at.timestamp(),
            status: user.status.clone(),
            projects_count: None,
        };
        if user.oauth_provider.contains("github") {
            u.social_name = Some(user.name.clone());
            u.social_provider = Some("github".to_string());
            u.social_link = Some(format!("https://github.com/{}", user.name));
        }
        u
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub value: String,
    pub name: String,
    pub created_at: i64,
    pub latest_used_at: i64,
    pub expired_at: i64,
    pub is_new: bool,
    pub id: i32,
}

impl Token {
    pub fn new(m: user_token::Model) -> Self {
        let expired_at = if let Some(expired_at) = m.expired_at {
            expired_at.timestamp()
        } else {
            0
        };
        let now = chrono::Utc::now().timestamp();
        let is_new = m.created_at.timestamp() + 30 > now;
        let mut token = Token {
            value: String::new(),
            name: m.name,
            created_at: m.created_at.timestamp(),
            latest_used_at: m.latest_used_at.timestamp(),
            expired_at,
            id: m.id,
            is_new,
        };
        if is_new {
            token.value = m.value;
        }
        token
    }
    pub fn new_from_models(models: Vec<user_token::Model>) -> Vec<Self> {
        models.into_iter().map(Token::new).collect()
    }
}
