use super::ServerError;
use crate::dashboard::{
    routers::HtmlMinified,
    templates::Engine,
    tplvars::{self, AuthUser, BreadCrumbKey, Page},
};
use axum::{response::IntoResponse, Extension};
use serde::Serialize;

/// index shows the dashboard page
pub async fn index(
    engine: Engine,
    Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub projects: Vec<tplvars::Project>,
    }
    let (projects_data, _) = land_dao::projects::list(Some(user.id), None, 1, 5).await?;
    Ok(HtmlMinified(
        "index.hbs",
        engine,
        Vars {
            page: Page::new("Dashboard", BreadCrumbKey::Dashboard, Some(user)),
            projects: tplvars::Project::new_from_models(projects_data, false).await?,
        },
    ))
}
