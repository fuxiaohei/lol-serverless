use super::Error;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Sha256, Sha384, Sha512};

type HmacSha1 = Hmac<Sha1>;
type HmacSha256 = Hmac<Sha256>;
type HmacSha384 = Hmac<Sha384>;
type HmacSha512 = Hmac<Sha512>;

macro_rules! sign_hmac {
    ($alg:ident, $secret:expr, $data:expr) => {{
        let mut mac = $alg::new_from_slice(&$secret).map_err(|_| Error::InvalidKey)?;
        mac.update(&$data);
        let result = mac.clone().finalize().into_bytes();
        Ok(result.to_vec())
    }};
}

/// sign sign data with secret
pub fn sign(hash: &str, secret: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>, Error> {
    match hash {
        "sha-1" => sign_hmac!(HmacSha1, secret, data),
        "sha-256" => sign_hmac!(HmacSha256, secret, data),
        "sha-384" => sign_hmac!(HmacSha384, secret, data),
        "sha-512" => sign_hmac!(HmacSha512, secret, data),
        _ => Err(Error::InvalidHash(hash.to_string())),
    }
}

macro_rules! verify_hmac {
    ($alg:ident, $secret:expr, $data:expr, $signature:expr) => {{
        let mut mac = $alg::new_from_slice(&$secret).map_err(|_| Error::InvalidKey)?;
        mac.update(&$data);
        let res = mac.clone().verify_slice(&$signature);
        Ok(res.is_ok())
    }};
}

/// verify verify signature with secret
pub fn verify(
    hash: &str,
    secret: Vec<u8>,
    data: Vec<u8>,
    signature: Vec<u8>,
) -> Result<bool, Error> {
    match hash {
        "sha-1" => verify_hmac!(HmacSha1, secret, data, signature),
        "sha-256" => verify_hmac!(HmacSha256, secret, data, signature),
        "sha-384" => verify_hmac!(HmacSha384, secret, data, signature),
        "sha-512" => verify_hmac!(HmacSha512, secret, data, signature),
        _ => Err(Error::InvalidHash(hash.to_string())),
    }
}
