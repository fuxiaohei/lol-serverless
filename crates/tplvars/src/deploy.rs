use land_dao::{models::deploys, settings::DomainSettings};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Deploy {
    pub id: i32,
    pub uuid: String,
    // short id is uuid first block for short display
    pub short_id: String,
    pub deploy_type: String,
    pub deploy_status: String,
    pub deploy_message: String,
    pub description: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub duration: i64,
    pub domain: String,
    pub domain_full: String,
    pub domain_url: String,
    pub is_current: bool,
}

impl Deploy {
    /// new_from_models creates a list of deploys from a list of models
    pub async fn new_from_models(models: Vec<deploys::Model>) -> anyhow::Result<Vec<Self>> {
        let mut deploys = Vec::new();
        let domain_settings = land_dao::settings::get_domain_settings().await?;
        for model in models {
            deploys.push(Deploy::new(&model, Some(domain_settings.clone())).await?);
        }
        // TODO: set is_current
        if !deploys.is_empty() {
            deploys[0].is_current = true;
        }
        Ok(deploys)
    }
    /// new creates a new deploy from a model
    pub async fn new(dp: &deploys::Model, ds: Option<DomainSettings>) -> anyhow::Result<Self> {
        let domain_settings = if let Some(ds) = ds {
            ds
        } else {
            land_dao::settings::get_domain_settings().await?
        };
        let prod_domain_full = format!("{}.{}", dp.domain, domain_settings.domain_suffix);
        let prod_domain_url = format!("{}://{}", domain_settings.http_protocol, prod_domain_full);
        let short_id = dp
            .task_id
            .split('-')
            .next()
            .ok_or_else(|| anyhow::anyhow!("task_id is empty"))?;
        let mut d = Deploy {
            id: dp.id,
            uuid: dp.task_id.clone(),
            short_id: short_id.to_string(),
            deploy_type: dp.deploy_type.clone(),
            deploy_status: dp.deploy_status.clone(),
            deploy_message: dp.deploy_message.clone(),
            description: dp.description.clone(),
            created_at: dp.created_at.and_utc().timestamp(),
            updated_at: dp.updated_at.and_utc().timestamp(),
            duration: 0,
            domain: dp.domain.clone(),
            domain_full: prod_domain_full.clone(),
            domain_url: prod_domain_url.clone(),
            is_current: false,
        };
        d.duration = d.updated_at - d.created_at;
        Ok(d)
    }
}
