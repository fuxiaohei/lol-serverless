use super::ServerError;
use crate::dashboard::{
    examples,
    routers::HtmlMinified,
    templates::Engine,
    tplvars::{AuthUser, BreadCrumbKey, Empty, Page, Vars},
};
use axum::{response::IntoResponse, Extension};
use serde::Serialize;

/// index shows the project dashboard page
pub async fn index(
    engine: Engine,
    Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, ServerError> {
    Ok(HtmlMinified(
        "projects.hbs",
        engine,
        Vars::new(
            Page::new("Projects", BreadCrumbKey::Projects, Some(user)),
            Empty::default(),
        ),
    ))
}

/// new is handler for projects new page, /new
pub async fn new(
    Extension(user): Extension<AuthUser>,
    engine: Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub examples: Vec<examples::Item>,
    }
    let examples = examples::defaults();
    Ok(HtmlMinified(
        "project-new.hbs",
        engine,
        Vars {
            page: Page::new("New Project", BreadCrumbKey::ProjectNew, Some(user)),
            examples,
        },
    ))
}
