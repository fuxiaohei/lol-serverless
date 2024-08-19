use anyhow::Result;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::Serialize;

/// rand_string generates a random string of the given size
pub fn rand_string(size: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

/// base64encode encode the given string
pub fn base64encode(value: Vec<u8>) -> String {
    BASE64_STANDARD.encode(value)
}

/// base64decode decode the given string
pub fn base64decode(value: &str) -> Result<Vec<u8>> {
    Ok(BASE64_STANDARD.decode(value)?)
}

/// obj_hash generate a hash of the given serializable object
pub fn obj_hash(value: impl Serialize) -> Result<String> {
    // serialize to json and md5 hash it
    let content = serde_json::to_string(&value)?;
    Ok(format!("{:x}", md5::compute(content)))
}

/// get hostname
pub fn get_hostname() -> Result<String> {
    // get env HOSTNAME first
    let mut h = std::env::var("HOSTNAME").unwrap_or_else(|_| "".to_string());
    if h.is_empty() {
        h = hostname::get().unwrap().to_str().unwrap().to_string();
    }
    Ok(h)
}

pub mod logging;
pub mod version;
