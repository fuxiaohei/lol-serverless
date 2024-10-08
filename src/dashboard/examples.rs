use crate::kernel::meta;
use anyhow::Result;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Assets is embedded examples assets
#[derive(RustEmbed)]
#[folder = "./examples"]
pub struct Assets;

/// Item is a example item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub link: String,
    pub title: String,
    pub description: String,
    pub asset_content: String,
    pub lang: String,
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.title, self.description)
    }
}

impl Item {
    /// get_source returns the source code of the example
    pub fn get_source(&self) -> Result<Option<String>> {
        let asset = Assets::get(&self.asset_content);
        if asset.is_none() {
            return Ok(None);
        }
        let asset = asset.unwrap();
        let content = std::str::from_utf8(asset.data.as_ref())?;
        Ok(Some(content.to_string()))
    }
    /// extract extracts the example to the target directory
    pub fn extract(&self, dir: &str, desc: &str) -> Result<()> {
        let meta_filename = format!("{}/{}", self.link, meta::DEFAULT_FILE);
        let meta_file = Assets::get(&meta_filename);
        if meta_file.is_none() {
            return Err(anyhow::anyhow!("Meta file not found"));
        }
        // extract template
        for item in Assets::iter() {
            if !item.starts_with(&self.link) {
                continue;
            }
            let raw_file = Assets::get(&item).unwrap();
            let target_file = format!("{}{}", dir, item.replace(&self.link, ""));
            let target_dir = std::path::Path::new(&target_file).parent().unwrap();
            if !target_dir.exists() {
                std::fs::create_dir_all(target_dir)?;
            }
            std::fs::write(&target_file, raw_file.data)?;
            debug!("Extract file: {}", target_file);
        }
        // refresh toml file
        let meta_desc = if desc.is_empty() {
            self.description.clone()
        } else {
            desc.to_string()
        };
        refresh_meta_toml(dir, &meta_desc)?;
        Ok(())
    }
}

fn refresh_meta_toml(dir: &str, desc: &str) -> Result<()> {
    let toml_file = format!("{}/{}", dir, meta::DEFAULT_FILE);
    let mut meta = meta::Data::from_file(&toml_file)?;
    meta.name = dir.to_string();
    meta.description = desc.to_string();
    meta.to_file(&toml_file)?;
    Ok(())
}

/// defaults return a list of default examples
pub fn defaults() -> Vec<Item> {
    vec![Item {
        link: "js-hello".to_string(),
        title: "Hello World - JavaScript".to_string(),
        description: "a simple hello world example by http trigger and return hello world string"
            .to_string(),
        asset_content: "js-hello/src/index.js".to_string(),
        lang: "javascript".to_string(),
    }]
}

/// get return a example by name
pub fn get(name: &str) -> Option<Item> {
    let examples = defaults();
    examples.into_iter().find(|example| example.link == name)
}
