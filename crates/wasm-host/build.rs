use land_wasm_gen::{generate_guest_code, GuestGeneratorType};
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=wit/*.wit");
    println!("cargo:rerun-if-changed=wit/deps/http/*.wit");
    println!("cargo:rerun-if-changed=wit/deps/asyncio/*.wit");

    build_wit_guest_code();
    copy_guest_code_to_sdk();
}

fn build_wit_guest_code() {
    let wit_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("wit");

    // set world name to parse. in Wit file, it can provide multiple worlds
    let worlds = vec!["http-handler", "http-service"];

    for world_name in worlds {
        let outputs = generate_guest_code(
            wit_dir.as_path(),
            Some(world_name.to_string()),
            GuestGeneratorType::Rust,
        )
        .unwrap_or_else(|err| panic!("Generate guest for {} failed: {:?}", world_name, err));

        // for range outputs, write content with key name
        for (name, content) in outputs.iter() {
            let target_rs = wit_dir.join(Path::new(name));
            std::fs::write(target_rs, content).unwrap();
        }
    }
}

fn copy_guest_code_to_sdk() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let wit_dir = manifest_dir.join("wit");
    let crates_dir = manifest_dir.parent().unwrap();
    let expects = [
        /*(
            "http_handler.rs",
            format!(
                "{}/wasm-impl/src/http_handler.rs",
                crates_dir.to_str().unwrap()
            ),
        ),*/
        (
            "http_handler.rs",
            format!(
                "{}/sdk-macro/src/http_handler.rs",
                crates_dir.to_str().unwrap()
            ),
        ),
        (
            "http_service.rs",
            format!("{}/sdk/src/http_service.rs", crates_dir.to_str().unwrap()),
        ),
    ];
    // copy expects
    for (source, target) in expects.iter() {
        let source_path = wit_dir.join(Path::new(source));
        let target_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(target);
        std::fs::copy(source_path, target_path).unwrap();
    }
}
