use crate::http_service::land::http::{body, types::BodyHandle};
use anyhow::{anyhow, Result};

/// Body is the body of the request or response
pub struct Body {
    /// The handle to the body
    body_handle: BodyHandle,
    /// Whether the body is is_writable or not,
    /// if it is not streaming, it means that the body is fully loaded in memory and not writable
    is_writable: bool,
}

impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Body")
            .field("body_handle", &self.body_handle)
            .field("is_writable", &self.is_writable)
            .finish()
    }
}

impl Body {
    pub fn empty() -> Self {
        let body_handle = body::new().unwrap();
        body::write(body_handle, "".as_bytes()).unwrap();
        Body {
            body_handle,
            is_writable: false,
        }
    }

    pub fn from_handle(body_handle: u32) -> Self {
        Self {
            body_handle,
            is_writable: false,
        }
    }
    pub fn body_handle(&self) -> u32 {
        self.body_handle
    }

    pub fn stream() -> Self {
        let body_handle = body::new_stream().unwrap();
        Body {
            body_handle,
            is_writable: true,
        }
    }

    pub fn read(&self, size: u32) -> Result<(Vec<u8>, bool)> {
        let resp = body::read(self.body_handle, size);
        Ok(resp.unwrap())
    }

    pub fn read_all(&self) -> Result<Vec<u8>> {
        match body::read_all(self.body_handle) {
            Ok(resp) => Ok(resp),
            Err(e) => Err(e.into()),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        self.read_all()
    }

    pub fn write(&self, data: &[u8]) -> Result<u64> {
        if !self.is_writable {
            return Err(anyhow!("body is not writable"));
        }
        let resp = body::write(self.body_handle, data);
        Ok(resp.unwrap())
    }

    pub fn write_str(&self, data: &str) -> Result<u64> {
        if !self.is_writable {
            return Err(anyhow!("body is not writable"));
        }
        let resp = body::write(self.body_handle, data.as_bytes());
        Ok(resp.unwrap())
    }

    pub fn is_writable(&self) -> bool {
        self.is_writable
    }
}

impl From<&[u8]> for Body {
    fn from(s: &[u8]) -> Self {
        let body_handle = body::new().unwrap();
        body::write(body_handle, s).unwrap();
        Body::from_handle(body_handle)
    }
}

impl From<&str> for Body {
    fn from(s: &str) -> Self {
        let body_handle = body::new().unwrap();
        body::write(body_handle, s.as_bytes()).unwrap();
        Body::from_handle(body_handle)
    }
}

impl From<String> for Body {
    fn from(s: String) -> Self {
        let body_handle = body::new().unwrap();
        body::write(body_handle, s.as_bytes()).unwrap();
        Body::from_handle(body_handle)
    }
}

impl From<Vec<u8>> for Body {
    fn from(v: Vec<u8>) -> Self {
        let body_handle = body::new().unwrap();
        body::write(body_handle, v.as_slice()).unwrap();
        Body::from_handle(body_handle)
    }
}
