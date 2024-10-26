use super::Error;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};

/// digest data with sha
pub fn digest(al: &str, data: Vec<u8>) -> Result<Vec<u8>, Error> {
    match al {
        "sha-1" => {
            let mut hasher = Sha1::new();
            hasher.update(data);
            Ok(hasher.finalize().to_vec())
        }
        "sha-256" => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            Ok(hasher.finalize().to_vec())
        }
        "sha-384" => {
            let mut hasher = Sha384::new();
            hasher.update(data);
            Ok(hasher.finalize().to_vec())
        }
        "sha-512" => {
            let mut hasher = Sha512::new();
            hasher.update(data);
            Ok(hasher.finalize().to_vec())
        }
        _ => Err(Error::InvalidAlgorithm(al.to_string())),
    }
}
