use futures::channel::mpsc;
use futures::stream::Stream;
use futures::stream::StreamExt;

use httpbis::ServerIncreaseInWindow;

use crate::result;

use crate::server::req_handler::ServerRequestStreamHandler;
use futures::task::Context;
use std::pin::Pin;
use std::task::Poll;

pub(crate) enum HandlerToStream<Req: Send + 'static> {
    Message(Req, u32),
    EndStream,
    Error(crate::Error),
    BufferProcessed(usize),
}

/// Easy to use request stream object.
///
/// This object implements [`Stream`].
///
/// This object increases incoming window after each message received.
pub struct ServerRequestStream<Req>
where
    Req: Send + 'static,
{
    pub(crate) req: mpsc::UnboundedReceiver<HandlerToStream<Req>>,
    pub(crate) increase_in_window: ServerIncreaseInWindow,
}

pub(crate) struct ServerRequestStreamSenderHandler<Req: Send + 'static> {
    pub(crate) sender: mpsc::UnboundedSender<HandlerToStream<Req>>,
}

impl<Req: Send + 'static> ServerRequestStreamSenderHandler<Req> {
    fn send(&mut self, message: HandlerToStream<Req>) -> result::Result<()> {
        // TODO: error
        Ok(self
            .sender
            .unbounded_send(message)
            .map_err(|_| crate::Error::Other("xxx"))?)
    }
}

impl<Req: Send + 'static> ServerRequestStreamHandler<Req>
    for ServerRequestStreamSenderHandler<Req>
{
    fn grpc_message(&mut self, message: Req, frame_size: u32) -> result::Result<()> {
        self.send(HandlerToStream::Message(message, frame_size))
    }

    fn end_stream(&mut self) -> result::Result<()> {
        self.send(HandlerToStream::EndStream)
    }

    fn error(&mut self, error: crate::Error) -> result::Result<()> {
        self.send(HandlerToStream::Error(error))
    }

    fn buffer_processed(&mut self, buffered: usize) -> result::Result<()> {
        self.send(HandlerToStream::BufferProcessed(buffered))
    }
}

impl<Req: Send + 'static> Stream for ServerRequestStream<Req> {
    type Item = crate::Result<Req>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<crate::Result<Req>>> {
        loop {
            // TODO: error
            let item = match self.req.poll_next_unpin(cx) {
                Poll::Ready(Some(r)) => r,
                Poll::Ready(None) => {
                    return Poll::Ready(Some(Err(crate::Error::Other("unexpected EOF"))))
                }
                Poll::Pending => return Poll::Pending,
            };

            match item {
                HandlerToStream::Message(req, frame_size) => {
                    // TODO: increase on next poll
                    self.increase_in_window.data_frame_processed(frame_size);
                    self.increase_in_window.increase_window_auto()?;
                    return Poll::Ready(Some(Ok(req)));
                }
                HandlerToStream::Error(error) => {
                    return Poll::Ready(Some(Err(error)));
                }
                HandlerToStream::BufferProcessed(buffered) => {
                    // TODO: overflow
                    self.increase_in_window
                        .increase_window_auto_above(buffered as u32)?;
                    continue;
                }
                HandlerToStream::EndStream => {
                    return Poll::Ready(None);
                }
            }
        }
    }
}
