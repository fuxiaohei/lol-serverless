use lazy_static::lazy_static;

/// build_info returns the version information of the current build.
pub fn build_info() -> String {
    format!(
        "{} ({} {})",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_GIT_SHA"),
        env!("VERGEN_BUILD_DATE"),
    )
}

lazy_static! {
    pub static ref VERSION: String = build_info();
}

/// get returns the version of the current build.
pub fn get() -> &'static str {
    &VERSION
}

/// get_about returns the version of the current build.
pub fn get_about() -> String {
    format!("{}\nThe Runtime.land command line tool", get())
}