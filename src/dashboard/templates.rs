use anyhow::Result;
use handlebars::Handlebars;
use rust_embed::RustEmbed;
use std::path::PathBuf;
use tracing::instrument;

/// Engine is the template engine for axum_template
pub type Engine = axum_template::engine::Engine<Handlebars<'static>>;

#[derive(RustEmbed)]
#[folder = "./templates"]
#[include = "*.hbs"]
#[include = "*.html"]
#[include = "*.css"]
#[include = "*.js"]
#[include = "*.png"]
#[include = "*.jpg"]
#[include = "*.ico"]
#[include = "*.svg"]
pub struct Assets;

/// new_handlebar creates a new handlebars instance with templates extracted to the static directory,
/// or load from the tpldir directory.
pub fn new_handlebar(dir: &str, tpl_dir: Option<String>) -> Result<Handlebars<'static>> {
    if let Some(tpl_dir) = tpl_dir {
        return init_handlebars(&tpl_dir);
    }
    extract(dir)?;
    init_handlebars(dir)
}

/// extract extracts all assets to the statis directory.
fn extract(dir: &str) -> Result<()> {
    Assets::iter().for_each(|file| {
        let filepath = file.to_string();

        let content = Assets::get(&filepath).unwrap().data;
        let mut path = PathBuf::from(dir);
        path.push(filepath);
        // debug!(path = path.to_str(), "Extract asset");

        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    });
    Ok(())
}

#[instrument("[TPL]")]
fn init_handlebars(dir: &str) -> Result<Handlebars<'static>> {
    let mut hbs = Handlebars::new();
    hbs.set_dev_mode(true);
    // hbs.register_helper("is_active", Box::new(is_active));

    // register templates
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let extension = path.extension().unwrap_or_default();
        if extension != "hbs" && extension != "html" {
            continue;
        }
        // get relative path from dir
        let tpl_name = path.strip_prefix(dir).unwrap().to_str().unwrap();
        // convert windows path slash to unix
        let tpl_name = tpl_name.replace('\\', "/");
        // debug!(name = tpl_name, "Register");
        hbs.register_template_file(&tpl_name, path)?;
    }
    Ok(hbs)
}
