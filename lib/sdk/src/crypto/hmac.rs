use super::{Algorithm, Error};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Sha256, Sha384, Sha512};
use std::collections::HashMap;
use std::sync::Mutex;

type HmacSha1 = Hmac<Sha1>;
type HmacSha256 = Hmac<Sha256>;
type HmacSha384 = Hmac<Sha384>;
type HmacSha512 = Hmac<Sha512>;

lazy_static::lazy_static! {
    static ref HMAC_HANDLES: Mutex<HashMap<u32, Algorithm>> = Mutex::new(HashMap::new());
    static ref HMAC_SHA1: Mutex<HashMap<u32,HmacSha1>> = Mutex::new(HashMap::new());
    static ref HMAC_SHA256: Mutex<HashMap<u32,HmacSha256>> = Mutex::new(HashMap::new());
    static ref HMAC_SHA384: Mutex<HashMap<u32,HmacSha384>> = Mutex::new(HashMap::new());
    static ref HMAC_SHA512: Mutex<HashMap<u32,HmacSha512>> = Mutex::new(HashMap::new());
}

macro_rules! new_hmac_key {
    ($alg:ident, $map:ident, $handle:expr, $key:expr) => {{
        let hmac = $alg::new_from_slice($key).map_err(|_| Error::InvalidKey)?;
        $map.lock().unwrap().insert($handle, hmac);
    }};
}

/// new_key create hmac key
pub fn new_key(handle: u32, al: Algorithm, key: &[u8]) -> Result<(), Error> {
    HMAC_HANDLES.lock().unwrap().insert(handle, al.clone());
    match al {
        Algorithm::HmacSha1 => new_hmac_key!(HmacSha1, HMAC_SHA1, handle, key),
        Algorithm::HmacSha256 => new_hmac_key!(HmacSha256, HMAC_SHA256, handle, key),
        Algorithm::HmacSha384 => new_hmac_key!(HmacSha384, HMAC_SHA384, handle, key),
        Algorithm::HmacSha512 => new_hmac_key!(HmacSha512, HMAC_SHA512, handle, key),
    }
    Ok(())
}

macro_rules! sign_hmac {
    ($alg:ident, $map:ident, $handle:expr, $data:expr) => {{
        let mut map = $map.lock().unwrap();
        let mac = map.get_mut(&$handle).ok_or(Error::InvalidHandle)?;
        mac.update($data);
        let result = mac.clone().finalize().into_bytes();
        Ok(result.to_vec())
    }};
}

/// sign sign data and return signature
pub fn sign(handle: u32, data: Vec<u8>) -> Result<Vec<u8>, Error> {
    let handles = HMAC_HANDLES.lock().unwrap();
    let al = handles.get(&handle).ok_or(Error::InvalidHandle)?;
    match al {
        Algorithm::HmacSha1 => sign_hmac!(HMAC_SHA1, HMAC_SHA1, handle, &data),
        Algorithm::HmacSha256 => sign_hmac!(HMAC_SHA256, HMAC_SHA256, handle, &data),
        Algorithm::HmacSha384 => sign_hmac!(HMAC_SHA384, HMAC_SHA384, handle, &data),
        Algorithm::HmacSha512 => sign_hmac!(HMAC_SHA512, HMAC_SHA512, handle, &data),
    }
}

macro_rules! verify_hmac {
    ($alg:ident, $map:ident, $handle:expr, $signature:expr, $data:expr) => {{
        let mut map = $map.lock().unwrap();
        let mac = map.get_mut(&$handle).ok_or(Error::InvalidHandle)?;
        mac.update($data);
        let res = mac.clone().verify_slice($signature);
        res.is_ok()
    }};
}

/// verify verify signature
pub fn verify(handle: u32, signature: Vec<u8>, data: Vec<u8>) -> Result<bool, Error> {
    let handles = HMAC_HANDLES.lock().unwrap();
    let al = handles.get(&handle).ok_or(Error::InvalidHandle)?;
    let res = match al {
        Algorithm::HmacSha1 => verify_hmac!(HMAC_SHA1, HMAC_SHA1, handle, &signature, &data),
        Algorithm::HmacSha256 => verify_hmac!(HMAC_SHA256, HMAC_SHA256, handle, &signature, &data),
        Algorithm::HmacSha384 => verify_hmac!(HMAC_SHA384, HMAC_SHA384, handle, &signature, &data),
        Algorithm::HmacSha512 => verify_hmac!(HMAC_SHA512, HMAC_SHA512, handle, &signature, &data),
    };
    Ok(res)
}
