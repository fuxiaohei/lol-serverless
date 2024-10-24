use anyhow::Result;
use serde::Serialize;

pub mod crypt;

/// obj_hash generate a hash of the given serializable object
pub fn obj_hash(value: impl Serialize) -> Result<String> {
    // serialize to json and md5 hash it
    let content = serde_json::to_string(&value)?;
    Ok(format!("{:x}", md5::compute(content)))
}
