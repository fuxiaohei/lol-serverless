use super::WasmConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TraefikConfs {
    pub http: HttpTraefikConfs,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpTraefikConfs {
    // pub services: HashMap<String, Service>,
    pub middlewares: BTreeMap<String, MiddlewareGroup>,
    pub routers: BTreeMap<String, Router>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MiddlewareGroup {
    pub headers: MiddlewareHeader,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MiddlewareHeader {
    // #[serde(rename = "customResponseHeaders")]
    // pub custom_response_headers: HashMap<String, String>,
    #[serde(rename = "customRequestHeaders")]
    pub custom_request_headers: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Router {
    // #[serde(rename = "entryPoints")]
    // pub entry_points: Vec<String>,
    pub middlewares: Vec<String>,
    pub service: String,
    pub rule: String,
}

/// build builds the TraefikConfs for the given TaskValue.
pub fn build(item: &WasmConfig, service_name: &str) -> Result<TraefikConfs> {
    let mut traefik_confs = HttpTraefikConfs {
        //services: HashMap::new(),
        routers: BTreeMap::new(),
        middlewares: BTreeMap::new(),
    };
    let mut headers = MiddlewareHeader {
        custom_request_headers: BTreeMap::new(),
    };
    // check filepath exist
    headers
        .custom_request_headers
        .insert("x-land-m".to_string(), item.file_name.clone());
    headers
        .custom_request_headers
        .insert("x-land-uid".to_string(), item.user_id.to_string());
    headers
        .custom_request_headers
        .insert("x-land-pid".to_string(), item.project_id.to_string());
    headers
        .custom_request_headers
        .insert("x-land-did".to_string(), item.deploy_id.to_string());
    traefik_confs
        .middlewares
        .insert(format!("m-{}", item.task_id), MiddlewareGroup { headers });

    let router = Router {
        middlewares: vec![format!("m-{}", item.task_id)],
        service: service_name.to_string(),
        rule: format!("Host(`{}`)", item.domain),
    };
    traefik_confs
        .routers
        .insert(format!("r-{}", item.task_id), router);
    Ok(TraefikConfs {
        http: traefik_confs,
    })
}
