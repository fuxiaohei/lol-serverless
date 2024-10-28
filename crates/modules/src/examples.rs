use anyhow::Result;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};

/// Assets is embedded examples assets
#[derive(RustEmbed)]
#[folder = "../../examples"]
struct Assets;

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
