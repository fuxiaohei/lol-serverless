use super::host::land::http::body::{self, BodyError, BodyHandle};
use super::HostContext;

#[async_trait::async_trait]
impl body::Host for HostContext {
    async fn read(&mut self, handle: BodyHandle, size: u32) -> Result<(Vec<u8>, bool), BodyError> {
        self.read_body(handle, size).await
    }

    async fn read_all(&mut self, handle: BodyHandle) -> Result<Vec<u8>, BodyError> {
        self.read_body_all(handle).await
    }

    async fn write(&mut self, handle: BodyHandle, data: Vec<u8>) -> Result<u64, BodyError> {
        self.write_body(handle, data).await
    }

    async fn new(&mut self) -> Result<BodyHandle, BodyError> {
        Ok(self.new_empty_body())
    }

    async fn new_stream(&mut self) -> Result<BodyHandle, BodyError> {
        Ok(self.new_writable_body())
    }
}
