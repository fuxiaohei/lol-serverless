use anyhow::Result;
use base64::{prelude::BASE64_STANDARD, Engine};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Serialize;
use std::collections::HashMap;

/// rand_string generates a random string of the given size
pub fn rand_string(size: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

/// obj_hash generate a hash of the given serializable object
pub fn obj_hash(value: impl Serialize) -> Result<String> {
    // serialize to json and md5 hash it
    let content = serde_json::to_string(&value)?;
    Ok(format!("{:x}", md5::compute(content)))
}

/// base64encode encode the given string
pub fn base64encode(value: Vec<u8>) -> String {
    BASE64_STANDARD.encode(value)
}

/// base64decode decode the given string
pub fn base64decode(value: &str) -> Result<Vec<u8>> {
    Ok(BASE64_STANDARD.decode(value)?)
}

/// decode map from secret and encrypt string
pub fn decode(secret: &str, encrypt_string: &str) -> Result<HashMap<String, String>> {
    let encrypt_data = base64decode(encrypt_string)?;
    let decrypt_data = simple_crypt::decrypt(&encrypt_data, secret.as_bytes())?;
    let q_map = serde_json::from_slice(&decrypt_data)?;
    Ok(q_map)
}
