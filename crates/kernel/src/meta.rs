use anyhow::Result;
use serde::{Deserialize, Serialize};

const VERSION: &str = "0.5";
pub const DEFAULT_FILE: &str = "land.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub name: String,
    pub description: String,
    pub language: String,
    pub version: String,
    pub build: BuildData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildData {
    pub main: String,
    pub cmd: Option<String>,
}

impl Data {
    pub fn new_js() -> Data {
        Data {
            name: "js".to_string(),
            description: "JavaScript".to_string(),
            language: "javascript".to_string(),
            version: VERSION.to_string(),
            build: BuildData {
                main: "src/index.js".to_string(),
                cmd: None,
            },
        }
    }
    pub fn from_file(file: &str) -> Result<Data> {
        let content = std::fs::read_to_string(file)?;
        let data: Data = toml::from_str(&content)?;
        Ok(data)
    }
    pub fn to_file(&self, file: &str) -> Result<()> {
        let content = toml::to_string(self)?;
        std::fs::write(file, content)?;
        Ok(())
    }
    pub fn target_wasm_path(&self) -> String {
        if self.language == "js" || self.language == "javascript" {
            return format!("dist/{}.wasm", self.name);
        }
        self.build.main.clone()
    }
}
