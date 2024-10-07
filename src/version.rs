fn build_short() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn build_long() -> String {
    format!(
        "{} ({} {})\ncommit Hash: {}\ncommit Date: {}\nrust version: {}",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_GIT_SHA"),
        env!("VERGEN_BUILD_DATE"),
        env!("VERGEN_GIT_SHA"),
        env!("VERGEN_GIT_COMMIT_DATE"),
        env!("VERGEN_RUSTC_SEMVER"),
    )
}

/// short version of the binary
pub fn short() -> String {
    build_short()
}

/// print the version of the binary
pub fn print(binary: &str, verbose: bool) {
    if verbose {
        println!("{} {}", binary, build_long());
    } else {
        println!("{} {}", binary, build_short());
    }
}
