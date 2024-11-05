use anyhow::Result;
use serde::{Deserialize, Serialize};

/// DEFAULT_FILE is the default file name for the meta file
pub const DEFAULT_FILE: &str = "land.toml";

/// Data is the meta data of the meta
#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub name: String,
    pub description: String,
    pub language: String,
    pub version: String,
    pub build: BuildData,
}

/// BuildData is the build data of the meta
#[derive(Debug, Serialize, Deserialize)]
pub struct BuildData {
    pub main: String,
    pub cmd: Option<String>,
}

impl Data {
    /// from_file reads the meta file from the file system
    pub fn from_file(file: &str) -> Result<Data> {
        let content = std::fs::read_to_string(file)?;
        let data: Data = toml::from_str(&content)?;
        Ok(data)
    }
    /// to_file writes the meta data to the file system
    pub fn to_file(&self, file: &str) -> Result<()> {
        let content = toml::to_string(self)?;
        std::fs::write(file, content)?;
        Ok(())
    }
    /// target_wasm_path returns the target wasm path by different language
    pub fn target_wasm_path(&self) -> String {
        if self.language == "js" || self.language == "javascript" {
            return format!("dist/{}.wasm", self.name);
        }
        self.build.main.clone()
    }
}
