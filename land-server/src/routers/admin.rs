use super::{
    setting::{handle_token_internal, TokenForm},
    HtmlMinified, ServerError,
};
use crate::{routers::utils::ok_htmx, templates::Engine};
use axum::{extract::Query, response::IntoResponse, Extension, Form};
use land_dao::{projects, settings, tokens, workers};
use land_service::storage;
use land_tplvars::{
    new_empty_admin, new_vars_admin, BreadCrumbKey, Pagination, Project, Token, Worker,
};
use serde::{Deserialize, Serialize};
use tracing::info;

/// index shows the admin page
pub async fn index(
    engine: Engine,
    Extension(user): Extension<land_tplvars::User>,
) -> Result<impl IntoResponse, ServerError> {
    Ok(HtmlMinified(
        "admin/index.hbs",
        engine,
        new_empty_admin("Admin", BreadCrumbKey::Admin, Some(user)),
    ))
}

/// FilterQuery is the query params for pager and filter
#[derive(Debug, Deserialize)]
pub struct FilterQuery {
    pub page: Option<u64>,
    pub size: Option<u64>,
    pub search: Option<String>,
}

/// projects shows the admin projects page
pub async fn projects(
    engine: Engine,
    Extension(user): Extension<land_tplvars::User>,
    Query(page): Query<FilterQuery>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Data {
        pub projects: Vec<Project>,
        pub pagination: Pagination,
    }
    let (current_page, current_size) = (page.page.unwrap_or(1), page.size.unwrap_or(20));
    let (projects, pager) =
        projects::list(None, page.search.clone(), current_page, current_size).await?;
    let pagination = Pagination::new(
        current_page,
        current_size,
        pager.number_of_pages,
        pager.number_of_items,
        "/admin/projects",
    );
    Ok(HtmlMinified(
        "admin/projects.hbs",
        engine,
        new_vars_admin(
            "Admin",
            BreadCrumbKey::AdminProjects,
            Some(user),
            Data {
                projects: Project::new_from_models(projects, true).await?,
                pagination,
            },
        ),
    ))
}

/// deploys shows the admin deploys page
pub async fn deploys(
    engine: Engine,
    Extension(user): Extension<land_tplvars::User>,
) -> Result<impl IntoResponse, ServerError> {
    Ok(HtmlMinified(
        "admin/index.hbs",
        engine,
        new_empty_admin("Admin", BreadCrumbKey::AdminDeploys, Some(user)),
    ))
}

/// general shows the admin general settings page
pub async fn general(
    engine: Engine,
    Extension(user): Extension<land_tplvars::User>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Data {
        pub domain_settings: land_dao::settings::DomainSettings,
        pub storage: land_service::storage::Vars,
    }
    let domain_settings = land_dao::settings::get_domain_settings().await?;
    Ok(HtmlMinified(
        "admin/general.hbs",
        engine,
        new_vars_admin(
            "Admin",
            BreadCrumbKey::AdminGeneral,
            Some(user),
            Data {
                domain_settings,
                storage: land_service::storage::Vars::get().await?,
            },
        ),
    ))
}

#[derive(Debug, Deserialize)]
pub struct UpdateDomainForm {
    pub domain: String,
    pub protocol: Option<String>,
}

/// handle_update_domains updates the domain settings, POST /admin/domains
pub async fn handle_update_domains(
    Form(f): Form<UpdateDomainForm>,
) -> Result<impl IntoResponse, ServerError> {
    info!("Update domain settings: {:?}", f);
    settings::set_domain_settings(&f.domain, &f.protocol.unwrap_or("http".to_string())).await?;
    Ok(ok_htmx("Updated successfully"))
}

/// handle_update_storage for admin storage, POST /admin/storage
pub async fn handle_update_storage(
    Form(form): Form<storage::Form>,
) -> Result<impl IntoResponse, ServerError> {
    storage::update_by_form(form).await?;
    Ok(ok_htmx("Storage updated"))
}

/// workers shows the workers page
pub async fn workers(
    engine: Engine,
    Extension(user): Extension<land_tplvars::User>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Data {
        pub tokens: Vec<land_tplvars::Token>,
        pub token_url: String,
        pub workers: Vec<Worker>,
    }

    let tokens = tokens::list(Some(user.id), Some(tokens::Usage::Worker)).await?;
    info!(
        owner_id = user.id,
        "List workers tokens, count: {}",
        tokens.len()
    );
    let tokens = Token::new_from_models(tokens);
    let workers_value = workers::find_all(None).await?;
    info!("List workers, count: {}", workers_value.len());
    let workers = workers_value.iter().map(Worker::new).collect();
    Ok(HtmlMinified(
        "admin/workers.hbs",
        engine,
        new_vars_admin(
            "Workers",
            BreadCrumbKey::AdminWorkers,
            Some(user),
            Data {
                tokens,
                token_url: "/admin/workers/tokens".to_string(),
                workers,
            },
        ),
    ))
}

/// handle_workers_token handles the workers token form
pub async fn handle_workers_token(
    Extension(user): Extension<land_tplvars::User>,
    Form(f): Form<TokenForm>,
) -> Result<impl IntoResponse, ServerError> {
    handle_token_internal(
        user.id,
        f,
        "/admin/workers",
        land_dao::tokens::Usage::Worker,
    )
    .await
}
