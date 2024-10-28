use super::{HtmlMinified, ServerError};
use crate::{routers::ok_htmx, templates::Engine};
use axum::{response::IntoResponse, Extension, Form};
use land_tplvars::BreadCrumbKey;
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
        land_tplvars::Empty::new_admin("Admin", BreadCrumbKey::Admin, Some(user)),
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
        pub storage: land_modules::storage::Vars,
    }
    let domain_settings = land_dao::settings::get_domain_settings().await?;
    Ok(HtmlMinified(
        "admin/general.hbs",
        engine,
        land_tplvars::Vars::new(
            land_tplvars::Page::new_admin("Admin", BreadCrumbKey::AdminGeneral, Some(user)),
            Data {
                domain_settings,
                storage: land_modules::storage::Vars::get().await?,
            },
        ),
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
    land_dao::settings::set_domain_settings(&f.domain, &f.protocol.unwrap_or("http".to_string()))
        .await?;
    Ok(ok_htmx("Updated successfully"))
}

/// update_storage for admin storage, POST /admin/storage
pub async fn update_storage(
    Form(form): Form<land_modules::storage::Form>,
) -> Result<impl IntoResponse, ServerError> {
    land_modules::storage::update_by_form(form).await?;
    Ok(ok_htmx("Storage updated"))
}
