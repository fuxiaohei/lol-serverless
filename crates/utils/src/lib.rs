pub mod crypt;
pub mod logger;
pub mod version;

/// get hostname
pub fn get_hostname() -> anyhow::Result<String> {
    // get env HOSTNAME first
    let mut h = std::env::var("HOSTNAME").unwrap_or_else(|_| "".to_string());
    if h.is_empty() {
        h = hostname::get().unwrap().to_str().unwrap().to_string();
    }
    Ok(h)
}
