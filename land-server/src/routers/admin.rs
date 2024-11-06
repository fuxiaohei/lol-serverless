use super::{HtmlMinified, ServerError};
use crate::{routers::utils::ok_htmx, templates::Engine};
use axum::{response::IntoResponse, Extension, Form};
use land_dao::settings;
use land_service::storage;
use land_tplvars::{new_empty_admin, new_vars_admin, BreadCrumbKey};
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
