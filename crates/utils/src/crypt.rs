use anyhow::Result;
use base64::{prelude::BASE64_STANDARD, Engine};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use simple_crypt::{decrypt, encrypt};
use std::collections::HashMap;

/// base64encode encode the given string
pub fn base64encode(value: Vec<u8>) -> String {
    BASE64_STANDARD.encode(value)
}

/// base64decode decode the given string
pub fn base64decode(value: &str) -> Result<Vec<u8>> {
    Ok(BASE64_STANDARD.decode(value)?)
}

/// rand_string generates a random string of the given size
pub fn rand_string(size: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

/// encode_map encode map to secret and encrypt string
pub fn encode_map(m: HashMap<String, String>) -> Result<(String, String)> {
    let q_value = serde_json::to_vec(&m)?;
    let secret = rand_string(12);
    let encrypt_data = encrypt(&q_value, secret.as_bytes())?;
    let encrypt_string = base64encode(encrypt_data);
    Ok((secret, encrypt_string))
}

/// encode_map_with_secret encode map to encrypt string
pub fn encode_map_with_secret(m: HashMap<String, String>, secret: &str) -> Result<String> {
    let q_value = serde_json::to_vec(&m)?;
    let encrypt_data = encrypt(&q_value, secret.as_bytes())?;
    let encrypt_string = base64encode(encrypt_data);
    Ok(encrypt_string)
}

/// decode map from secret and encrypt string
pub fn decode(secret: &str, encrypt_string: &str) -> Result<HashMap<String, String>> {
    let encrypt_data = base64decode(encrypt_string)?;
    let decrypt_data = decrypt(&encrypt_data, secret.as_bytes())?;
    let q_map = serde_json::from_slice(&decrypt_data)?;
    Ok(q_map)
}
