use axum::body::{Body, Bytes};
use futures_util::Future;
use http_body::Frame;
use http_body_util::BodyExt;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::{mpsc, oneshot};
use super::host_service::land::http::body::BodyError;

#[derive(Debug)]
pub enum FinishMessage {
    Finished,
    _Abort,
}

type BodyReceiver = mpsc::Receiver<Bytes>;
type FinishReceiver = oneshot::Receiver<FinishMessage>;
type FinishSender = oneshot::Sender<FinishMessage>;

pub struct ChannelBodyImpl {
    body_receiver: BodyReceiver,
    finish_receiver: Option<FinishReceiver>,
}

impl http_body::Body for ChannelBodyImpl {
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
                        FinishMessage::_Abort => {
                            Poll::Ready(Some(Err(BodyError::ReadFailed("abort".to_string()))))
                        }
                    },
                }
            }
        }
    }
}

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

    pub fn _abort(mut self) -> Result<(), BodyError> {
        drop(self.writer); // drop writer to notify receiver
        let finish_sender = self.finish_sender.take().expect("finish_sender is illgal");
        let _ = finish_sender.send(FinishMessage::_Abort);
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

pub fn new_channel() -> (Sender, Body) {
    let (body_sender, body_receiver) = mpsc::channel(3);
    let (finish_sender, finish_receiver) = oneshot::channel();

    let body_impl = ChannelBodyImpl {
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
