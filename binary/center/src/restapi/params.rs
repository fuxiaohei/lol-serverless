use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct SignupEmailRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 4))]
    pub nickname: String,
}

#[derive(Serialize, Debug)]
pub struct LoginResponse {
    pub token_value: String,
    pub token_uuid: String,
    pub token_expired_at: i64,
    pub nick_name: String,
    pub email: String,
    pub avatar_url: String,
    pub oauth_id: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct LoginEmailRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct LoginTokenRequest {
    #[validate(length(min = 12))]
    pub token: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct CreateTokenRequest {
    pub name: String,
    pub display_name: String,
    pub email: String,
    pub image_url: String,
    pub oauth_id: String,
    pub oauth_provider: String,
    pub oauth_social: String,
}