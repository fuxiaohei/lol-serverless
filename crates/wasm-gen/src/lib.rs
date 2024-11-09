use anyhow::{anyhow, Result};
use std::{
    collections::HashMap,
    env::{current_dir, current_exe},
    io::Write,
    process::{Command, Stdio},
};
use tracing::debug;
use wasi_preview1_component_adapter_provider::WASI_SNAPSHOT_PREVIEW1_REACTOR_ADAPTER;
use wit_bindgen_core::{wit_parser::Resolve, Files, WorldGenerator};
use wit_bindgen_rust::Opts;
use wit_component::ComponentEncoder;

// GuestGeneratorType is the type of the guest generator.
pub enum GuestGeneratorType {
    Rust,
}

impl GuestGeneratorType {
    /// create a new guest generator
    fn create(&self) -> Result<Box<dyn WorldGenerator>> {
        match self {
            GuestGeneratorType::Rust => {
                let opts = Opts {
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

/// generate_guest_code parse wit file and generate guest code
pub fn generate_guest_code(
    wit_dir: &std::path::Path,
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

fn find_cmd(cmd: &str) -> Result<std::path::PathBuf> {
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

/// compile_js compile js file to wasm module
fn compile_js(src_path: &str, dst_path: &str, js_engine: Option<String>) -> Result<()> {
    debug!("Compile js file: {}", src_path);
    let cmd = find_cmd("wizer")?;
    let dir = std::path::Path::new(src_path).parent().unwrap();
    let js_engine_file = dir.join("js_engine.wasm");

    // copy js_engine.wasm to path dir
    let js_engine_bytes = if let Some(js_engine) = js_engine {
        debug!("Read js_engine: {}", js_engine);
        std::fs::read(js_engine)?
    } else {
        debug!("Read js_engine from embedded");
        include_bytes!("../engine/js-engine.wasm").to_vec()
    };
    debug!("Read js_engine_bytes: {}", js_engine_bytes.len());
    std::fs::write(&js_engine_file, js_engine_bytes)?;

    let src_content = std::fs::read(src_path)?;

    // call wizer to compile js to wasm
    // wizer js_engine.wasm -o {path}.wasm --allow-wasi --inherit-stdio=true --inherit-env=true
    let mut child = Command::new(cmd)
        .arg(&js_engine_file)
        .arg("-o")
        .arg(dst_path)
        .arg("--allow-wasi")
        .arg("--inherit-stdio=true")
        .arg("--inherit-env=true")
        .arg("--wasm-bulk-memory=true")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute wizer child process");
    let mut stdin = child.stdin.take().expect("Failed to get stdin");

    std::thread::spawn(move || {
        stdin
            .write_all(src_content.as_slice())
            .expect("Failed to write to stdin");
    });

    let output = child
        .wait_with_output()
        .expect("Failed to wait on wizer child process");
    if !output.status.success() {
        let err = String::from_utf8(output.stderr)?;
        return Err(anyhow!(err));
    }
    // print output
    let std_out_string = String::from_utf8_lossy(&output.stdout);
    let std_out_string2 = std_out_string.trim_end();
    debug!("Wizer output:\n{}", std_out_string2);
    if std_out_string2 != "success" {
        return Err(anyhow!("{}", std_out_string2));
    }
    debug!("Wizer success, from {} to {}", src_path, dst_path);
    let _ = std::fs::remove_file(&js_engine_file);
    Ok(())
}

/// optimize wasm module by wasm-opt
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

/// componentize_wasm compile wasm to wasm component
pub fn componentize_wasm(target: &str) -> Result<()> {
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

/// componentize_js compile to js to wasm component
pub fn componentize_js(src: &str, target: &str, js_engine: Option<String>) -> Result<()> {
    // compile js to wizer
    compile_js(src, target, js_engine)?;
    componentize_wasm(target)
}
