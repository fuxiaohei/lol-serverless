use super::{
    ok_html,
    settings::{handle_token_internal, TokenForm},
    ServerError,
};
use crate::dashboard::{
    routers::HtmlMinified,
    templates::Engine,
    tplvars::{self, AuthUser, BreadCrumbKey, Page},
};
use axum::{response::IntoResponse, Extension, Form};
use land_dao::{
    settings::{self, DomainSettings},
    tokens,
};
use land_kernel::storage;
use serde::{Deserialize, Serialize};
use tracing::info;

/// index shows the admin page
pub async fn index(
    engine: Engine,
    Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: tplvars::Page,
        pub domain_settings: DomainSettings,
        pub storage: storage::Vars,
    }
    let domain_settings = settings::get_domain_settings().await?;
    Ok(HtmlMinified(
        "admin.hbs",
        engine,
        Vars {
            page: Page::new_admin("Administration", BreadCrumbKey::AdminOverview, Some(user)),
            domain_settings,
            storage: storage::Vars::get().await?,
        },
    ))
}

#[derive(Debug, Deserialize)]
pub struct UpdateDomainForm {
    pub domain: String,
    pub protocol: Option<String>,
}

/// update_domains updates the domain settings, POST /admin/domains
pub async fn update_domains(
    Form(f): Form<UpdateDomainForm>,
) -> Result<impl IntoResponse, ServerError> {
    info!("Update domain settings: {:?}", f);
    settings::set_domain_settings(&f.domain, &f.protocol.unwrap_or("http".to_string())).await?;
    Ok(ok_html("Updated successfully"))
}

/// update_storage for admin storage, POST /admin/storage
pub async fn update_storage(
    Form(form): Form<storage::Form>,
) -> Result<impl IntoResponse, ServerError> {
    storage::update_by_form(form).await?;
    Ok(ok_html("Storage updated"))
}

/// general shows the admin general settings page
pub async fn general(
    engine: Engine,
    Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: tplvars::Page,
        pub domain_settings: DomainSettings,
        pub storage: storage::Vars,
    }
    let domain_settings = settings::get_domain_settings().await?;
    Ok(HtmlMinified(
        "admin/general.hbs",
        engine,
        Vars {
            page: Page::new_admin("Administration", BreadCrumbKey::AdminGeneral, Some(user)),
            domain_settings,
            storage: storage::Vars::get().await?,
        },
    ))
}

/// workers shows the workers page
pub async fn workers(
    engine: Engine,
    Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Data {
        pub tokens: Vec<tplvars::Token>,
        pub token_url: String,
        pub workers: Vec<tplvars::Worker>,
    }

    let tokens = tokens::list(Some(user.id), Some(tokens::Usage::Worker)).await?;
    info!(
        owner_id = user.id,
        "List workers tokens, count: {}",
        tokens.len()
    );
    let tokens = tplvars::Token::new_from_models(tokens);
    let workers_value = land_dao::workers::find_all(None).await?;
    info!("List workers, count: {}", workers_value.len());
    let workers = workers_value.iter().map(tplvars::Worker::new).collect();
    Ok(HtmlMinified(
        "admin/workers.hbs",
        engine,
        tplvars::Vars::new(
            Page::new_admin("Workers", BreadCrumbKey::AdminWorkers, Some(user)),
            Data {
                tokens,
                token_url: "/admin/workers/tokens".to_string(),
                workers,
            },
        ),
    ))
}

/// handle_token handles the token form
pub async fn handle_token(
    Extension(user): Extension<AuthUser>,
    Form(f): Form<TokenForm>,
) -> Result<impl IntoResponse, ServerError> {
    handle_token_internal(user, f, "/admin/workers", tokens::Usage::Worker).await
}
