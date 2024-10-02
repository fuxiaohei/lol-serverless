use land_wasm_gen::{generate_guest, GuestGeneratorType};
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wit/*.wit");
    println!("cargo:rerun-if-changed=wit/deps/landhttp/*.wit");
    println!("cargo:rerun-if-changed=wit/deps/asyncio/*.wit");

    build_wit_guest_code();
    // copy_guest_code_to_sdk();
}

fn build_wit_guest_code() {
    let wit_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("wit");

    // set world name to parse. in Wit file, it can provide multiple worlds
    let worlds = vec!["http-handler", "http-service"];

    for world_name in worlds {
        let outputs = generate_guest(
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
