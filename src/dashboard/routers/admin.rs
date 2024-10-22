use super::{ok_html, ServerError};
use crate::dashboard::{
    routers::HtmlMinified,
    templates::Engine,
    tplvars::{self, AuthUser, BreadCrumbKey, Page},
};
use axum::{response::IntoResponse, Extension, Form};
use land_dao::settings::{self, DomainSettings};
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
    }
    let domain_settings = settings::get_domain_settings().await?;
    Ok(HtmlMinified(
        "admin.hbs",
        engine,
        Vars {
            page: Page::new("Administration", BreadCrumbKey::Administration, Some(user)),
            domain_settings,
        },
    ))
}

#[derive(Debug, Deserialize)]
pub struct UpdateDomainForm {
    pub domain: String,
    pub protocol: Option<String>,
}

/// update_domains updates the domain settings, /admin/domains
pub async fn update_domains(
    Form(f): Form<UpdateDomainForm>,
) -> Result<impl IntoResponse, ServerError> {
    info!("Update domain settings: {:?}", f);
    settings::set_domain_settings(&f.domain, &f.protocol.unwrap_or("http".to_string())).await?;
    Ok(ok_html("Updated successfully"))
}
