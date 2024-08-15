use crate::{
    dash::{ok_html, ServerError},
    templates::{Engine, RenderHtmlMinified},
};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use land_core::traffic;
use land_dao::{deploys, playground, projects};
use land_vars::{AuthUser, BreadCrumbKey, Page, Pagination, Project};
use serde::{Deserialize, Serialize};
use tracing::info;

pub async fn index(
    Extension(user): Extension<AuthUser>,
    engine: Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub nav_admin: bool,
        pub projects: Vec<Project>,
        pub pagination: Pagination,
    }

    let (project_models, pager) = projects::list(None, None, 1, 20).await?;
    let projects = Project::new_from_models(project_models, true).await?;
    let pagination = Pagination::new(
        1,
        20,
        pager.number_of_pages,
        pager.number_of_items,
        "/admin/projects",
    );

    Ok(RenderHtmlMinified(
        "admin/projects.hbs",
        engine,
        Vars {
            nav_admin: true,
            page: Page::new("Admin Projects", BreadCrumbKey::AdminProjects, Some(user)),
            projects,
            pagination,
        },
    ))
}

/// flows is route of traffic requests query page, /admin/projects/traffic/
pub async fn traffic(
    Json(f): Json<traffic::ProjectsQueryForm>,
) -> Result<impl IntoResponse, ServerError> {
    let now = tokio::time::Instant::now();
    let pids = f
        .pids
        .iter()
        .map(|pid| pid.to_string())
        .collect::<Vec<String>>();
    let period = traffic::PeriodParams::new(&f.period, None);
    let lines = traffic::projects_traffic(None, pids, &period).await?;
    info!(
        "admin-projects, start:{}, end:{}, step:{}, cost:{}",
        period.start,
        period.end,
        period.step,
        now.elapsed().as_millis(),
    );
    Ok(Json(lines))
}

#[derive(Deserialize, Debug)]
pub struct SourceQuery {
    pub pid: i32,
}

/// source is route of show playground source page, /admin/projects/source
pub async fn source(Query(query): Query<SourceQuery>) -> Result<impl IntoResponse, ServerError> {
    let playground = playground::get_by_project(query.pid).await?;
    if playground.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Playground not found",
        ));
    }
    Ok(playground.unwrap().source.into_response())
}

/// details is route of project detail page, /admin/projects/:name
pub async fn details(
    Extension(user): Extension<AuthUser>,
    engine: Engine,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub nav_admin: bool,
        pub project_admin_name: String,
        pub project: Project,
    }
    let project = projects::get_by_name(&name, None).await?;
    if project.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Project not found",
        ));
    }
    let project_model = project.unwrap();
    let mut project = Project::new_with_owner(&project_model).await?;
    let p2 = Project::new_with_source(&project_model).await?;
    project.source = p2.source;
    Ok(RenderHtmlMinified(
        "admin/project-details.hbs",
        engine,
        Vars {
            nav_admin: true,
            page: Page::new(
                format!("Admin Project - {}", project.name).as_str(),
                BreadCrumbKey::AdminProjects,
                Some(user),
            ),
            project_admin_name: name,
            project,
        },
    ))
}

/// redeploy is route of project redeploy page, /admin/projects/:name/redeploy
pub async fn redeploy(
    Extension(_user): Extension<AuthUser>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    let project = projects::get_by_name(&name, None).await?;
    if project.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Project not found",
        ));
    }
    let project = project.unwrap();
    let deploy = projects::create_deploy(&project, deploys::DeployType::Production).await?;
    info!(
        "admin-projects, redeploy project:{}, dp-id:{}",
        project.name, deploy.id
    );
    Ok(ok_html("Project redeployed"))
}

/// enable is route of project enable page, /admin/projects/:name/enable
pub async fn enable(
    Extension(_user): Extension<AuthUser>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    let project = projects::get_by_name(&name, None).await?;
    if project.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Project not found",
        ));
    }
    let project = project.unwrap();
    // set project status to active
    projects::set_status(project.id, projects::Status::Active).await?;
    let deploy = projects::create_deploy(&project, deploys::DeployType::Production).await?;
    info!(
        "admin-projects, enable project:{}, dp-id:{}",
        project.name, deploy.id,
    );
    Ok(ok_html("Project enabled"))
}

/// disable is route of project disable page, /admin/projects/:name/disable
pub async fn disable(
    Extension(_user): Extension<AuthUser>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    let project = projects::get_by_name(&name, None).await?;
    if project.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Project not found",
        ));
    }
    let project = project.unwrap();
    // set project status to active
    projects::set_status(project.id, projects::Status::Disabled).await?;
    let deploy = projects::create_deploy(&project, deploys::DeployType::Disabled).await?;
    info!(
        "admin-projects, disable project:{}, dp-id:{}",
        project.name, deploy.id,
    );
    Ok(ok_html("Project disabled"))
}
