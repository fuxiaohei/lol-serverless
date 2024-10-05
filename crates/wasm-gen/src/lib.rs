use anyhow::{anyhow, Result};
use wit_component::ComponentEncoder;
use std::{
    collections::HashMap,
    env::{current_dir, current_exe},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use tracing::debug;
use wasi_preview1_component_adapter_provider::WASI_SNAPSHOT_PREVIEW1_REACTOR_ADAPTER;
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
                    // export_macro_name: Some("http_export".to_string()),
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

/// componentize compile wasm to wasm component
pub fn componentize(target: &str) -> Result<()> {
    // use wasm-opt to optimize wasm if wasm-opt exists
    if let Some(op) = optimize(target)? {
        std::fs::rename(op, target)?;
    }

    // encode wasm module to component
    encode_component(target, target)?;

    // check target exists
    if !std::path::Path::new(target).exists() {
        return Err(anyhow::anyhow!(
            "Build target '{}' does not exist!",
            &target,
        ));
    }
    Ok(())
}

fn find_cmd(cmd: &str) -> Result<PathBuf> {
    let c = match which::which(cmd) {
        Ok(c) => c,
        Err(_) => {
            // find xxx binary in current work directroy ./xxx/xxx
            let exe_path = current_dir()?;
            let file = exe_path.parent().unwrap().join(format!("{}/{}", cmd, cmd));

            #[cfg(target_os = "windows")]
            let file = file.with_extension("exe");

            debug!("Try find cmd: {:?}", file);
            if file.exists() {
                return Ok(file);
            }

            // find xxx binary in current executable file
            let exe_path2 = current_exe()?;
            let file2 = exe_path2.parent().unwrap().join(format!("{}/{}", cmd, cmd));

            #[cfg(target_os = "windows")]
            let file2 = file2.with_extension("exe");

            debug!("Try find cmd: {:?}", file2);
            if file2.exists() {
                return Ok(file2);
            }

            return Err(anyhow!(
                "cannot find '{}' binary, it should in $PATH or {:?} or {:?}",
                cmd,
                file,
                file2,
            ));
        }
    };
    Ok(c)
}

/// optimize wasm component
pub fn optimize(path: &str) -> Result<Option<String>> {
    let cmd = match find_cmd("wasm-opt") {
        Ok(cmd) => cmd,
        Err(_err) => {
            return Ok(None);
        }
    };
    let target = path.replace(".wasm", ".opt.wasm");
    let child = Command::new(cmd)
        .arg("-O3") // use O3 instead of --strip-debug, https://github.com/fastly/js-compute-runtime/commit/dd91fa506b74487b70dc5bec510e89de95e1c569
        // .arg("--strip-debug")
        .arg("-o")
        .arg(&target)
        .arg(path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to execute wasm-opt child process");
    let output = child
        .wait_with_output()
        .expect("Failed to wait on wasm-opt child process");
    if !output.status.success() {
        let err = String::from_utf8(output.stderr)?;
        return Err(anyhow::anyhow!(err));
    }
    debug!("Wasm-opt success, from {} to {}", path, target);
    let _ = std::fs::remove_file(path);
    Ok(Some(target))
}

/// encode_component encode wasm module file to component
fn encode_component(src: &str, dest: &str) -> Result<()> {
    let file_bytes = std::fs::read(src)?;
    let wasi_adapter = WASI_SNAPSHOT_PREVIEW1_REACTOR_ADAPTER;
    let component = ComponentEncoder::default()
        .module(&file_bytes)
        .expect("Pull custom sections from module")
        .validate(true)
        .adapter("wasi_snapshot_preview1", wasi_adapter)
        .expect("Add adapter to component")
        .encode()
        .expect("Encode component");
    let output = src.replace(".wasm", ".component.wasm");
    std::fs::write(&output, component)?;
    debug!("Convert component success, from {} to {}", src, dest);
    // remove *.component.wasm temp file
    if output != dest {
        std::fs::rename(&output, dest)?;
        let _ = std::fs::remove_file(output);
    }
    Ok(())
}
