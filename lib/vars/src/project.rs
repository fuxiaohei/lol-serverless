use crate::AuthUser;
use anyhow::{anyhow, Result};
use land_dao::{deploys, models::project, playground, projects::CreatedBy, settings};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Project {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub prod_domain: String,
    pub prod_domain_full: String,
    pub prod_domain_url: String,
    pub dev_domain: String,
    pub dev_domain_full: String,
    pub dev_domain_url: String,
    pub description: String,
    pub language: String,
    pub created_by: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub status: String,
    pub deploy_status: String,
    pub deploy_message: String,
    pub is_deploy_success: bool,
    pub is_editable: bool,
    pub source: Option<String>,
    pub owner_id: i32,
    pub owner: Option<AuthUser>,
}

impl Project {
    /// new_from_models creates a list of projects from a list of models
    pub async fn new_from_models(models: Vec<project::Model>) -> Result<Vec<Self>> {
        let mut projects = Vec::new();
        for model in models {
            projects.push(Project::new(&model).await?);
        }
        Ok(projects)
    }
    /// new creates a new project from a model
    pub async fn new(project: &project::Model) -> anyhow::Result<Self> {
        let domain_settings = settings::get_domain_settings().await?;
        let prod_domain_full = format!("{}.{}", project.prod_domain, domain_settings.domain_suffix);
        let prod_domain_url = format!("{}://{}", domain_settings.http_protocol, prod_domain_full);
        let dev_domain_full = format!("{}.{}", project.dev_domain, domain_settings.domain_suffix);
        let dev_domain_url = format!("{}://{}", domain_settings.http_protocol, dev_domain_full);
        Ok(Project {
            id: project.id,
            uuid: project.uuid.clone(),
            name: project.name.clone(),
            prod_domain: project.prod_domain.clone(),
            prod_domain_full,
            prod_domain_url,
            dev_domain: project.dev_domain.clone(),
            dev_domain_full,
            dev_domain_url,
            description: project.description.clone(),
            language: project.language.clone(),
            created_by: project.created_by.clone(),
            created_at: project.created_at.and_utc().timestamp(),
            updated_at: project.updated_at.and_utc().timestamp(),
            status: project.status.clone(),
            deploy_status: project.deploy_status.clone(),
            deploy_message: project.deploy_message.clone(),
            is_deploy_success: project.deploy_status == deploys::Status::Success.to_string(),
            is_editable: project.created_by == CreatedBy::Playground.to_string(),
            source: None,
            owner_id: project.owner_id,
            owner: None,
        })
    }

    /// new_with_source creates a new project from a model with playground source
    pub async fn new_with_source(project: &project::Model) -> anyhow::Result<Self> {
        let mut project = Project::new(project).await?;
        if project.created_by != CreatedBy::Playground.to_string() {
            return Err(anyhow!("Project is not created by playground"));
        }
        let playground = playground::get_by_project(project.id).await?;
        if playground.is_none() {
            return Err(anyhow!("Playground not found"));
        }
        project.source = Some(playground.unwrap().source);
        Ok(project)
    }
}
