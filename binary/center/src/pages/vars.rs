use crate::settings;

use super::auth::SessionUser;
use anyhow::Result;
use land_dao::deployment::{self, Status};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PageVars {
    pub title: String,
    pub base_uri: String,
    pub version: String,
    pub build_time: String,
}

impl PageVars {
    pub fn new(title: String, base_uri: String) -> Self {
        Self {
            title,
            base_uri,
            version: land_core::version::get_full().to_string(),
            build_time: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UserVars {
    pub name: String,
    pub email: String,
    pub avatar: String,
}

impl UserVars {
    pub fn new(user: &SessionUser) -> Self {
        Self {
            name: user.name.clone(),
            email: user.email.clone(),
            avatar: user.avatar.clone(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectVars {
    pub name: String,
    pub language: String,
    pub uuid: String,
    pub deployments: usize,
    pub production_url: String,
    pub production_label: String,
    pub deployment_url: String,
    pub deployment_label: String,
    pub updated_timeago: String,
    pub status_label: String,
    pub prod_domain: String,
    pub prod_protocol: String,
}

impl ProjectVars {
    pub async fn from_model(project: &land_dao::Project) -> Result<ProjectVars> {
        let (prod_domain, prod_protocol) = settings::get_domains().await;
        let tago = timeago::Formatter::new();
        let duration = chrono::Utc::now().signed_duration_since(project.updated_at);
        let mut project_vars = ProjectVars {
            name: project.name.clone(),
            language: project.language.clone(),
            uuid: project.uuid.clone(),
            deployments: 0,
            deployment_url: "".to_string(),
            deployment_label: "".to_string(),
            production_url: "".to_string(),
            production_label: "".to_string(),
            updated_timeago: tago.convert(duration.to_std().unwrap()),
            status_label: "".to_string(),
            prod_domain: prod_domain.clone(),
            prod_protocol: prod_protocol.clone(),
        };
        if project.prod_deploy_id > 0 {
            project_vars.production_url =
                format!("{}://{}.{}", prod_protocol, project.name, prod_domain);
            project_vars.production_label = format!("{}.{}", project.name, prod_domain);
            let deployment = deployment::find_by_id(project.owner_id, project.prod_deploy_id)
                .await?
                .unwrap();
            project_vars.deployment_url =
                format!("{}://{}.{}", prod_protocol, deployment.domain, prod_domain);
            project_vars.deployment_label = format!("{}.{}", deployment.domain, prod_domain);
        }
        Ok(project_vars)
    }

    pub async fn from_models(
        projects: &Vec<land_dao::Project>,
        counters: HashMap<i32, usize>,
    ) -> Result<Vec<ProjectVars>> {
        let (prod_domain, prod_protocol) = settings::get_domains().await;
        let tago = timeago::Formatter::new();
        let mut vars = vec![];
        for project in projects {
            let counter = counters.get(&project.id).unwrap_or(&0);
            let duration = chrono::Utc::now().signed_duration_since(project.updated_at);
            let mut project_vars = ProjectVars {
                name: project.name.clone(),
                language: project.language.clone(),
                uuid: project.uuid.clone(),
                deployments: *counter,
                production_url: "".to_string(),
                deployment_url: "".to_string(),
                deployment_label: "".to_string(),
                production_label: "".to_string(),
                updated_timeago: tago.convert(duration.to_std().unwrap()),
                status_label: "running".to_string(),
                prod_domain: prod_domain.clone(),
                prod_protocol: prod_protocol.clone(),
            };
            if project.prod_deploy_id > 0 {
                project_vars.production_url =
                    format!("{}://{}.{}", prod_protocol, project.name, prod_domain);
                project_vars.production_label = format!("{}.{}", project.name, prod_domain)
            } else {
                project_vars.status_label = "develop".to_string();
            }
            if *counter == 0 {
                project_vars.status_label = "empty".to_string();
            }
            vars.push(project_vars);
        }
        Ok(vars)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeploymentVars {
    pub domain: String,
    pub prod_domain: String,
    pub deployment_url: String,
    pub deployment_label: String,
    pub is_prod: bool,
    pub is_enabled: bool,
    pub status: String,
    pub deploy_status: String,
    pub updated_timeago: String,
    pub uuid: String,
}

impl DeploymentVars {
    pub async fn from_models(
        deployments: &Vec<land_dao::Deployment>,
    ) -> Result<Vec<DeploymentVars>> {
        let (prod_domain, prod_protocol) = settings::get_domains().await;
        let tago = timeago::Formatter::new();
        let mut vars = vec![];
        for deployment in deployments {
            let duration = chrono::Utc::now().signed_duration_since(deployment.updated_at);
            let deployment_vars = DeploymentVars {
                domain: deployment.domain.clone(),
                prod_domain: deployment.prod_domain.clone(),
                is_prod: !deployment.prod_domain.is_empty(),
                status: deployment.status.clone(),
                deploy_status: deployment.deploy_status.clone(),
                updated_timeago: tago.convert(duration.to_std().unwrap()),
                deployment_url: format!(
                    "{}://{}.{}",
                    prod_protocol, deployment.domain, prod_domain
                ),
                deployment_label: format!("{}.{}", deployment.domain, prod_domain),
                is_enabled: deployment.status == Status::Active.to_string(),
                uuid: deployment.uuid.clone(),
            };
            vars.push(deployment_vars);
        }
        Ok(vars)
    }
}
