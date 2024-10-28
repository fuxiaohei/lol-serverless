use super::{HtmlMinified, ServerError};
use crate::templates::Engine;
use axum::{response::IntoResponse, Extension};
use land_modules::examples;
use land_tplvars::{BreadCrumbKey, Empty};
use serde::Serialize;

/// index shows the projects page
pub async fn index(
    engine: Engine,
    Extension(user): Extension<land_tplvars::User>,
) -> Result<impl IntoResponse, ServerError> {
    Ok(HtmlMinified(
        "projects.hbs",
        engine,
        Empty::new_vars("Projects", BreadCrumbKey::Projects, Some(user)),
    ))
}

/// new is handler for projects new page, /new
pub async fn new(
    Extension(user): Extension<land_tplvars::User>,
    engine: Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Data {
        pub examples: Vec<examples::Item>,
    }
    let examples = examples::defaults();
    Ok(HtmlMinified(
        "new.hbs",
        engine,
        land_tplvars::Vars::new(
            land_tplvars::Page::new("New Project", BreadCrumbKey::ProjectNew, Some(user)),
            Data { examples },
        ),
    ))
}
