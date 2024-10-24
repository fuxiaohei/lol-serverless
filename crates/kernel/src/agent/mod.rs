#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WasmConfig {
    pub user_id: i32,
    pub project_id: i32,
    pub deploy_id: i32,
    pub task_id: String,
    pub file_name: String,
    pub download_url: String,
    pub file_hash: String,
    pub domain: String,
    pub content: Option<String>,
}
