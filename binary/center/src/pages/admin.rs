use super::auth::SessionUser;
use super::vars::{
    DeployAdminVars, PageVars, PaginationVars, RuntimeNodeVars, StorageVars, UserAdminVars,
    UserVars,
};
use super::AppEngine;
use crate::pages::vars::ProjectAdminVars;
use axum::extract::Query;
use axum::response::{IntoResponse, Redirect};
use axum::{Extension, Form};
use axum_csrf::CsrfToken;
use axum_template::RenderHtml;
use hyper::StatusCode;
use land_dao::{deployment, project, user};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct AdminProjectVars {
    pub page: PageVars,
    pub user: UserVars,
    pub project_count: u64,
    pub projects: Vec<ProjectAdminVars>,
    pub pagination: PaginationVars,
    pub search: String,
    pub csrf_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectsQueryParams {
    pub page: Option<u64>,
    pub size: Option<u64>,
    pub search: Option<String>,
}

pub async fn render_projects(
    engine: AppEngine,
    csrf_token: CsrfToken,
    Extension(current_user): Extension<SessionUser>,
    Query(query): Query<ProjectsQueryParams>,
) -> impl IntoResponse {
    let csrf_token_value = csrf_token.authenticity_token().unwrap();
    let page = query.page.unwrap_or(1);
    let page_size = query.size.unwrap_or(20);
    let (projects, pages, alls) =
        project::list_all_available_with_page(query.search.clone(), page, page_size)
            .await
            .unwrap();

    let project_ids: HashSet<i32> = projects.iter().map(|p| p.id).collect();
    let deploy_counts = deployment::list_counter_by_projects(project_ids.into_iter().collect())
        .await
        .unwrap();

    let owner_ids: HashSet<i32> = projects.iter().map(|p| p.owner_id).collect();
    let owners = user::list_by_ids(owner_ids.into_iter().collect())
        .await
        .unwrap();

    let page_vars = PageVars::new(
        "Admin - Projects".to_string(),
        "/admin/projects".to_string(),
    );
    let user_vars = UserVars::new(&current_user);
    let project_vars = ProjectAdminVars::from_models(&projects, deploy_counts, owners)
        .await
        .unwrap();
    let pagination_vars = PaginationVars::new(page, pages, "/admin/projects");

    (
        csrf_token,
        RenderHtml(
            "admin/projects.hbs",
            engine,
            AdminProjectVars {
                page: page_vars,
                user: user_vars,
                project_count: alls,
                projects: project_vars,
                pagination: pagination_vars,
                search: query.search.unwrap_or_default(),
                csrf_token: csrf_token_value,
            },
        ),
    )
        .into_response()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HandleProjectParams {
    pub csrf_token: String,
    pub uuid: String,
    pub owner_id: i32,
    pub action: String,
    pub name: String,
}

pub async fn handle_project(
    csrf_token: CsrfToken,
    Form(payload): Form<HandleProjectParams>,
) -> Result<Redirect, StatusCode> {
    let action = payload.action.as_str();
    let name = payload.name.as_str();
    let span = tracing::info_span!("handle_project", action, name);
    let _enter = span.enter();

    if csrf_token.verify(&payload.csrf_token).is_err() {
        warn!("csrf token verify failed");
        return Err(StatusCode::BAD_REQUEST);
    }
    let project = match project::find_by_uuid(payload.uuid, payload.owner_id).await {
        Ok(p) => {
            if p.is_none() {
                warn!("project not found");
                return Err(StatusCode::NOT_FOUND);
            }
            p.unwrap()
        }
        Err(err) => {
            warn!("project found error, err:{}", err);
            return Err(StatusCode::NOT_FOUND);
        }
    };
    match payload.action.as_str() {
        "enable" => {
            project::set_active(project.id).await.unwrap();
        }
        "disable" => {
            project::set_inactive(project.id).await.unwrap();
        }
        _ => {}
    }
    info!("project action success");
    Ok(Redirect::to("/admin/projects"))
}

#[derive(Debug, Serialize, Deserialize)]
struct AdminDeploymentsVars {
    pub page: PageVars,
    pub user: UserVars,
    pub pagination: PaginationVars,
    pub search: String,
    pub deploys_count: u64,
    pub deploys: Vec<DeployAdminVars>,
    pub csrf_token: String,
}

pub async fn render_deployments(
    engine: AppEngine,
    Extension(current_user): Extension<SessionUser>,
    csrf_token: CsrfToken,
    Query(query): Query<ProjectsQueryParams>,
) -> impl IntoResponse {
    let csrf_token_value = csrf_token.authenticity_token().unwrap();
    let page = query.page.unwrap_or(1);
    let page_size = query.size.unwrap_or(20);
    let (deployments, pages, alls) =
        deployment::list_all_available_with_page(query.search.clone(), page, page_size)
            .await
            .unwrap();

    let owner_ids: HashSet<i32> = deployments.iter().map(|p| p.owner_id).collect();
    let owners = user::list_by_ids(owner_ids.into_iter().collect())
        .await
        .unwrap();

    let project_ids: HashSet<i32> = deployments.iter().map(|p| p.project_id).collect();
    let projects = project::list_by_ids(project_ids.into_iter().collect())
        .await
        .unwrap();

    let deploy_vars = DeployAdminVars::from_models(&deployments, projects, owners)
        .await
        .unwrap();

    let page_vars = PageVars::new(
        "Admin - Deployments".to_string(),
        "/admin/deployments".to_string(),
    );
    let user_vars = UserVars::new(&current_user);
    (
        csrf_token,
        RenderHtml(
            "admin/deployments.hbs",
            engine,
            AdminDeploymentsVars {
                page: page_vars,
                user: user_vars,
                pagination: PaginationVars::new(page, pages, "/admin/deployments"),
                search: query.search.unwrap_or_default(),
                deploys_count: alls,
                deploys: deploy_vars,
                csrf_token: csrf_token_value,
            },
        ),
    )
        .into_response()
}

type HandleDeployParams = HandleProjectParams;

pub async fn handle_deploy(
    csrf_token: CsrfToken,
    Form(payload): Form<HandleDeployParams>,
) -> Result<Redirect, StatusCode> {
    let action = payload.action.as_str();
    let name = payload.name.as_str();
    let span = tracing::info_span!("handle_deploy", action, name);
    let _enter = span.enter();

    if csrf_token.verify(&payload.csrf_token).is_err() {
        warn!("csrf token verify failed");
        return Err(StatusCode::BAD_REQUEST);
    }
    let deploy = match deployment::find_by_uuid(payload.owner_id, payload.uuid).await {
        Ok(p) => {
            if p.is_none() {
                warn!("deployment not found");
                return Err(StatusCode::NOT_FOUND);
            }
            p.unwrap()
        }
        Err(err) => {
            warn!("deployment found error,err:{}", err);
            return Err(StatusCode::NOT_FOUND);
        }
    };
    match payload.action.as_str() {
        "enable" => {
            deployment::enable(deploy.owner_id, deploy.uuid)
                .await
                .unwrap();
        }
        "disable" => {
            deployment::disable(deploy.owner_id, deploy.uuid)
                .await
                .unwrap();
        }
        _ => {}
    }
    info!("deployment action success");
    Ok(Redirect::to("/admin/deployments"))
}

#[derive(Debug, Serialize, Deserialize)]
struct AdminUsersVars {
    pub page: PageVars,
    pub user: UserVars,
    pub user_count: u64,
    pub users: Vec<UserAdminVars>,
    pub pagination: PaginationVars,
}

pub async fn render_users(
    engine: AppEngine,
    Extension(current_user): Extension<SessionUser>,
    Query(query): Query<ProjectsQueryParams>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);
    let page_size = query.size.unwrap_or(20);
    let (users, pages, alls) = user::list_with_page(query.search.clone(), page, page_size)
        .await
        .unwrap();

    let user_ids: Vec<i32> = users.iter().map(|u| u.id).collect();
    let deploys_counts = deployment::list_counter_by_owners(user_ids.clone())
        .await
        .unwrap();
    let projects_counts = project::list_counter_by_owners(user_ids).await.unwrap();

    let users_vars = UserAdminVars::from_models(&users, projects_counts, deploys_counts)
        .await
        .unwrap();

    let page_vars = PageVars::new("Admin - Users".to_string(), "/admin/users".to_string());
    let user_vars = UserVars::new(&current_user);
    RenderHtml(
        "admin/users.hbs",
        engine,
        AdminUsersVars {
            page: page_vars,
            user: user_vars,
            user_count: alls,
            users: users_vars,
            pagination: PaginationVars::new(page, pages, "/admin/users"),
        },
    )
}

#[derive(Debug, Serialize, Deserialize)]
struct AdminEndpointsVars {
    pub page: PageVars,
    pub user: UserVars,
    pub node_count: u64,
    pub nodes: Vec<RuntimeNodeVars>,
}

pub async fn render_runtime_nodes(
    engine: AppEngine,
    Extension(current_user): Extension<SessionUser>,
) -> impl IntoResponse {
    let page_vars = PageVars::new(
        "Runtime Nodes | Admin ".to_string(),
        "/admin/runtime-nodes".to_string(),
    );
    let user_vars = UserVars::new(&current_user);
    let nodes = land_dao::runtime_node::list_all().await.unwrap();
    let node_vars = RuntimeNodeVars::from_models(&nodes);
    RenderHtml(
        "admin/runtime_nodes.hbs",
        engine,
        AdminEndpointsVars {
            page: page_vars,
            user: user_vars,
            node_count: nodes.len() as u64,
            nodes: node_vars,
        },
    )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminStorageVars {
    pub page: PageVars,
    pub user: UserVars,
    pub storage: StorageVars,
}

pub async fn render_storage(
    engine: AppEngine,
    csrf_token: CsrfToken,
    Extension(current_user): Extension<SessionUser>,
) -> impl IntoResponse {
    let csrf_token_value = csrf_token.authenticity_token().unwrap();
    let page_vars = PageVars::new("Storage | Admin ".to_string(), "/admin/storage".to_string());
    let user_vars = UserVars::new(&current_user);
    let mut storage_vars = StorageVars::load().await.unwrap();
    storage_vars.csrf_token = csrf_token_value.clone();
    (
        csrf_token,
        RenderHtml(
            "admin/storage.hbs",
            engine,
            AdminStorageVars {
                page: page_vars,
                user: user_vars,
                storage: storage_vars,
            },
        ),
    )
        .into_response()
}

pub async fn handle_storage(
    csrf_token: CsrfToken,
    Form(payload): Form<StorageVars>,
) -> Result<Redirect, StatusCode> {
    let span = tracing::info_span!("handle_storage");
    let _enter = span.enter();

    if csrf_token.verify(&payload.csrf_token).is_err() {
        warn!("csrf token verify failed");
        return Err(StatusCode::BAD_REQUEST);
    }

    let (storage_type, fs, s3) = payload.to_model();

    // save configs
    fs.save_db().await.unwrap();
    s3.save_db().await.unwrap();
    // save storage type
    land_storage::dao::save_storage_type(storage_type.clone())
        .await
        .unwrap();

    info!("update success, storage_type:{}", storage_type);
    Ok(Redirect::to("/admin/storage"))
}
