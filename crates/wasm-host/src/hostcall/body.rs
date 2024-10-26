use super::host::land::http::body::BodyError;
use axum::body::{Body, BodyDataStream};
use bytes::Bytes;
use futures_util::StreamExt;
use http_body::Frame;
use http_body_util::BodyExt;
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::atomic::AtomicU32,
    task::{Context, Poll},
};
use tokio::sync::{mpsc, oneshot};

// READ_DEFAULT_SIZE is the default read size in once read if not specified
const READ_DEFAULT_SIZE: u32 = 128 * 1024;

/// BodyContext is a context for managing bodies.
pub struct BodyContext {
    body_seq_id: AtomicU32,
    body_map: HashMap<u32, Body>,
    body_buffer_map: HashMap<u32, Vec<u8>>,
    body_stream_map: HashMap<u32, BodyDataStream>,
    body_sender_map: HashMap<u32, Sender>,
    body_sender_closed: HashMap<u32, bool>,
}

impl BodyContext {
    /// new body context
    pub fn new() -> Self {
        Self {
            body_seq_id: AtomicU32::new(1),
            body_map: HashMap::new(),
            body_buffer_map: HashMap::new(),
            body_stream_map: HashMap::new(),
            body_sender_map: HashMap::new(),
            body_sender_closed: HashMap::new(),
        }
    }
    /// new_empty creates new empty body and returns handle id
    pub fn new_empty(&self) -> u32 {
        self.body_seq_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
    /// set_body sets body by id, it will return handle id
    pub fn set_body(&mut self, id: u32, body: Body) -> u32 {
        let handle = if id < 1 {
            self.body_seq_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        } else {
            id
        };
        self.body_map.insert(handle, body);
        handle
    }

    /// take_body takes body by id, it will remove body from map
    pub fn take_body(&mut self, id: u32) -> Option<Body> {
        self.body_map.remove(&id)
    }

    /// read_body reads body by id
    pub async fn read_body(
        &mut self,
        handle: u32,
        size: u32,
    ) -> Result<(Vec<u8>, bool), BodyError> {
        let read_size = if size == 0 { READ_DEFAULT_SIZE } else { size };
        let mut current_buffer = self.body_buffer_map.remove(&handle).unwrap_or_default();

        // if buffer is over the read size, split it and return the read part
        if current_buffer.len() > read_size as usize {
            let (read, rest) = current_buffer.split_at(read_size as usize);
            self.body_buffer_map.insert(handle, rest.to_vec());
            return Ok((read.to_vec(), false));
        }

        // if handle is Body, move it to BodyStream to read chunk
        if let Some(body) = self.body_map.remove(&handle) {
            let stream = body.into_data_stream();
            self.body_stream_map.insert(handle, stream);
        }

        // if handle is not in BodyStream, return InvalidHandle
        let stream = self
            .body_stream_map
            .get_mut(&handle)
            .ok_or(BodyError::InvalidHandle)?;

        loop {
            let chunk = stream.next().await;
            if chunk.is_none() {
                // no more data, no rest buffer
                // return empty vec and true to indicate end of stream
                if current_buffer.is_empty() {
                    // TODO: all data is read, set sender closed
                    // self.set_sender_closed(handle);
                    return Ok((vec![], true));
                }
                // return rest buffer
                return Ok((current_buffer, false));
            }
            let chunk = chunk.unwrap();
            let chunk = chunk.map_err(|err| {
                BodyError::ReadFailed(format!("Read body chunk failed: {:?}", err))
            })?;
            current_buffer.extend_from_slice(&chunk);
            if current_buffer.len() > read_size as usize {
                let (read, rest) = current_buffer.split_at(read_size as usize);
                self.body_buffer_map.insert(handle, rest.to_vec());
                return Ok((read.to_vec(), false));
            }
        }
    }

    /// set_sender_closed makes the body sender is closed.
    fn set_sender_closed(&mut self, handle: u32) {
        if self.body_sender_map.contains_key(&handle) {
            // call finish to notify receiver
            let sender = self.body_sender_map.remove(&handle).unwrap();
            let _ = sender.finish();
        }
        self.body_sender_closed.insert(handle, true);
    }

    /// read_body_all reads all body by id
    pub async fn read_body_all(&mut self, handle: u32) -> Result<Vec<u8>, BodyError> {
        // if read all, set sender closed to do not write more data
        self.set_sender_closed(handle);
        let (body, _) = self.read_body(handle, u32::MAX).await?;
        Ok(body)
    }

    /// new_writable_body creates new body stream and returns handle id
    pub fn new_writable_body(&mut self) -> u32 {
        let (sender, body) = super::body::new_channel();
        let handle = self.set_body(0, body);
        self.body_sender_map.insert(handle, sender);
        handle
    }

    /// write_body is used to write data to body
    pub async fn write_body(&mut self, handle: u32, data: Vec<u8>) -> Result<u64, BodyError> {
        let closed = self
            .body_sender_closed
            .get(&handle)
            .copied()
            .unwrap_or_default();
        if closed {
            return Err(BodyError::WriteClosed);
        }

        let data_len = data.len() as u64;
        // if Sender exist, write data to sender
        if self.body_sender_map.contains_key(&handle) {
            let sender = self.body_sender_map.get_mut(&handle).unwrap();
            sender.write(bytes::Bytes::from(data))?;
            return Ok(data_len);
        }

        // if exist in body map, return ReadOnly error
        if self.body_map.contains_key(&handle) {
            return Err(BodyError::ReadOnly);
        }

        // create new body but readonly
        let body = Body::from(data);
        self.set_body(handle, body);
        Ok(data_len)
    }
}

#[derive(Debug)]
enum FinishMessage {
    Finished,
}

type BodyReceiver = mpsc::Receiver<Bytes>;
type FinishReceiver = oneshot::Receiver<FinishMessage>;
type FinishSender = oneshot::Sender<FinishMessage>;

struct ChannelBody {
    body_receiver: BodyReceiver,
    finish_receiver: Option<FinishReceiver>,
}

impl http_body::Body for ChannelBody {
    type Data = Bytes;
    type Error = BodyError;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        use tokio::sync::oneshot::error::RecvError;

        match self.as_mut().body_receiver.poll_recv(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(frame)) => Poll::Ready(Some(Ok(Frame::data(frame)))),
            // This means that the `body_sender` end of the channel has been dropped.
            Poll::Ready(None) => {
                if self.finish_receiver.is_none() {
                    return Poll::Ready(None);
                }
                let mut finish_receiver = self.as_mut().finish_receiver.take().unwrap();
                match Pin::new(&mut finish_receiver).poll(cx) {
                    Poll::Pending => {
                        self.as_mut().finish_receiver = Some(finish_receiver);
                        Poll::Pending
                    }
                    Poll::Ready(Err(RecvError { .. })) => Poll::Ready(None),
                    Poll::Ready(Ok(message)) => match message {
                        FinishMessage::Finished => Poll::Ready(None),
                    },
                }
            }
        }
    }
}

/// Sender is a sender to send bytes to body with channel body.
#[derive(Debug)]
pub struct Sender {
    pub writer: mpsc::Sender<Bytes>,
    finish_sender: Option<FinishSender>,
}

impl Sender {
    pub fn finish(mut self) -> Result<(), BodyError> {
        drop(self.writer); // drop writer to notify receiver
        let finish_sender = self.finish_sender.take().expect("finish_sender is illgal");
        let _ = finish_sender.send(FinishMessage::Finished);
        Ok(())
    }

    pub fn write(&mut self, bytes: Bytes) -> Result<(), BodyError> {
        let res = self.writer.try_send(bytes);
        match res {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => {
                Err(BodyError::WriteFailed("channel full".to_string()))
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                Err(BodyError::WriteFailed("channel closed".to_string()))
            }
        }
    }
}

/// new_channel creates a new channel body and sender.
pub fn new_channel() -> (Sender, Body) {
    let (body_sender, body_receiver) = mpsc::channel(3);
    let (finish_sender, finish_receiver) = oneshot::channel();
    let body_impl = ChannelBody {
        body_receiver,
        finish_receiver: Some(finish_receiver),
    }
    .boxed();
    let body = Body::new(body_impl);
    let sender = Sender {
        writer: body_sender,
        finish_sender: Some(finish_sender),
    };
    (sender, body)
}
