mod hmac;
pub use hmac::sign as hmac_sign;
pub use hmac::verify as hmac_verify;

mod sha;
pub use sha::digest as sha_digest;

#[derive(Debug)]
/// Error is error for crypto
pub enum Error {
    /// InvalidHandle means handle is not exist or not match algorithm
    InvalidHandle,
    /// InvalidAlgorithm is invalid algorithm or not support
    InvalidAlgorithm(String),
    /// InvalidKey means crypto key is invalid
    InvalidKey,
    /// InvalidHash means hash is not support in this algorithm
    InvalidHash(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidAlgorithm(al) => write!(f, "invalid algorithm: {}", al),
            Error::InvalidKey => write!(f, "invalid key"),
            Error::InvalidHandle => write!(f, "invalid handle"),
            Error::InvalidHash(hash) => write!(f, "invalid hash: {}", hash),
        }
    }
}

/*
/// Algorithm are algorithms for crypto
#[derive(Debug, Clone, PartialEq)]
pub enum Algorithm {
    /// HmacSha1 is hmac-sha1
    HmacSha1,
    /// HmacSha256 is hmac-sha256
    HmacSha256,
    /// HmacSha384 is hmac-sha384
    HmacSha384,
    /// HmacSha512 is hmac-sha512
    HmacSha512,
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Algorithm::HmacSha1 => write!(f, "hmac-sha1"),
            Algorithm::HmacSha256 => write!(f, "hmac-sha256"),
            Algorithm::HmacSha384 => write!(f, "hmac-sha384"),
            Algorithm::HmacSha512 => write!(f, "hmac-sha512"),
        }
    }
}

struct Context {
    handle_seq: AtomicU32,
    handles: HashMap<u32, Algorithm>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            handle_seq: AtomicU32::new(1),
            handles: HashMap::new(),
        }
    }
    pub fn new_handle(&mut self, al: Algorithm) -> u32 {
        let handle = self
            .handle_seq
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.handles.insert(handle, al);
        handle
    }
}

fn convert_algorithm(al: &str) -> Result<Algorithm, Error> {
    match al {
        "hmac-sha1" => Ok(Algorithm::HmacSha1),
        "hmac-sha256" => Ok(Algorithm::HmacSha256),
        "hmac-sha384" => Ok(Algorithm::HmacSha384),
        "hmac-sha512" => Ok(Algorithm::HmacSha512),
        _ => Err(Error::InvalidAlgorithm(al.to_string())),
    }
}

/// import_key import crypto key
pub fn import_key(al: &str, key: Vec<u8>) -> Result<u32, Error> {
    let al = convert_algorithm(al)?;
    let handle = CTX.lock().unwrap().new_handle(al.clone());
    if al == Algorithm::HmacSha1
        || al == Algorithm::HmacSha256
        || al == Algorithm::HmacSha384
        || al == Algorithm::HmacSha512
    {
        return Ok(handle);
    }
    Err(Error::InvalidAlgorithm(al.to_string()))
}

/// sign sign data with handle that index to key
pub fn sign(handle: u32, data: Vec<u8>) -> Result<Vec<u8>, Error> {
    let al = CTX.lock().unwrap().handles.get(&handle).unwrap().clone();
    if al == Algorithm::HmacSha1
        || al == Algorithm::HmacSha256
        || al == Algorithm::HmacSha384
        || al == Algorithm::HmacSha512
    {}
    Err(Error::InvalidAlgorithm(al.to_string()))
}

/// verify verify signature with handle that index to key
pub fn verify(handle: u32, signature: Vec<u8>, data: Vec<u8>) -> Result<bool, Error> {
    let al = CTX.lock().unwrap().handles.get(&handle).unwrap().clone();
    if al == Algorithm::HmacSha1
        || al == Algorithm::HmacSha256
        || al == Algorithm::HmacSha384
        || al == Algorithm::HmacSha512
    {}
    Err(Error::InvalidAlgorithm(al.to_string()))
}
*/
