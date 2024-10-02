use anyhow::Result;
use std::{collections::HashMap, path::Path};
use wit_bindgen_core::{wit_parser::Resolve, Files, WorldGenerator};

// GuestGeneratorType is the type of the guest generator.
pub enum GuestGeneratorType {
    Rust,
}

impl GuestGeneratorType {
    /// create a new guest generator
    fn create(&self) -> Result<Box<dyn WorldGenerator>> {
        match self {
            GuestGeneratorType::Rust => {
                let opts = wit_bindgen_rust::Opts {
                    // exports,
                    format: true,
                    generate_all: true,
                    pub_export_macro: true,
                    ..Default::default()
                };
                let builder = opts.build();
                Ok(builder)
            } // _ => Err(anyhow!("Unsupport guest generator")),
        }
    }
}

/// generate_guest parse wit file and return world id
pub fn generate_guest(
    wit_dir: &Path,
    world: Option<String>,
    t: GuestGeneratorType,
) -> Result<HashMap<String, String>> {
    let mut generator = t.create()?;

    let mut resolve = Resolve::default();
    let pkg = resolve.push_dir(wit_dir)?.0;

    let mut output_maps = HashMap::new();
    let mut files = Files::default();
    let world = resolve.select_world(pkg, world.as_deref())?;
    generator.generate(&resolve, world, &mut files)?;
    for (name, contents) in files.iter() {
        output_maps.insert(
            name.to_string(),
            String::from_utf8_lossy(contents).to_string(),
        );
    }
    Ok(output_maps)
}
