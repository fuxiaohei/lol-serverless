use super::{error_html, hx_redirect, ServerError};
use crate::dashboard::{
    routers::HtmlMinified,
    templates::Engine,
    tplvars::{self, AuthUser, BreadCrumbKey, Page, Vars},
};
use anyhow::anyhow;
use axum::{response::IntoResponse, Extension, Form};
use land_dao::tokens;
use serde::{Deserialize, Serialize};
use tracing::info;

/// index shows the settings page
pub async fn index(
    engine: Engine,
    Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Data {
        pub tokens: Vec<tplvars::Token>,
        pub token_url: String,
    }

    let tokens = tokens::list(Some(user.id), Some(tokens::Usage::Cmdline)).await?;
    info!(owner_id = user.id, "List tokens, count: {}", tokens.len());
    let tokens = tplvars::Token::new_from_models(tokens);

    Ok(HtmlMinified(
        "settings.hbs",
        engine,
        Vars::new(
            Page::new("Settings", BreadCrumbKey::Settings, Some(user)),
            Data {
                tokens,
                token_url: "/settings/tokens".to_string(),
            },
        ),
    ))
}

/// handle_token handles the token form
pub async fn handle_token(
    Extension(user): Extension<AuthUser>,
    Form(f): Form<TokenForm>,
) -> Result<impl IntoResponse, ServerError> {
    handle_token_internal(user, f, "/settings", tokens::Usage::Cmdline).await
}

/// TokenForm is the form for creating and removing a new token
#[derive(Deserialize, Debug)]
pub struct TokenForm {
    pub name: String,
    pub op: String,
    pub id: Option<i32>,
}

/// handle_token_internal create a new token handler for user or remove a token
pub async fn handle_token_internal(
    user: AuthUser,
    f: TokenForm,
    redirect: &str,
    usage: tokens::Usage,
) -> Result<impl IntoResponse, ServerError> {
    if f.op == "create" {
        match create_token(user, f, usage).await {
            Ok(_) => Ok(hx_redirect(redirect).into_response()),
            Err(e) => Ok(error_html(&e.to_string()).into_response()),
        }
    } else if f.op == "remove" {
        match remove_token(user, f, usage).await {
            Ok(_) => Ok(hx_redirect(redirect).into_response()),
            Err(e) => Ok(error_html(&e.to_string()).into_response()),
        }
    } else {
        Ok(error_html("Invalid operation").into_response())
    }
}

/// create_token create a new token for user
async fn create_token(user: AuthUser, f: TokenForm, usage: tokens::Usage) -> anyhow::Result<()> {
    let exist_token = tokens::get_by_name(&f.name, user.id, Some(usage.clone())).await?;
    if exist_token.is_some() {
        return Err(anyhow!("Token name already exists"));
    }
    let token = tokens::create(user.id, &f.name, 3600 * 24 * 365, usage).await?;

    info!(
        owner_id = user.id,
        token_name = f.name,
        "Create new token: {:?}",
        token
    );
    Ok(())
}

/// remove_token removes a token
async fn remove_token(user: AuthUser, f: TokenForm, usage: tokens::Usage) -> anyhow::Result<()> {
    let token = tokens::get_by_name(&f.name, user.id, Some(usage)).await?;
    if token.is_none() {
        return Err(anyhow!("Token not found"));
    }
    let token = token.unwrap();
    if token.id != f.id.unwrap_or(0) {
        return Err(anyhow!("Token id not match"));
    }
    tokens::set_expired(token.id, &f.name).await?;
    info!(
        owner_id = user.id,
        token_name = f.name,
        "Remove token: {}",
        token.id,
    );
    Ok(())
}
