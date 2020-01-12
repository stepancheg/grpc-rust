use error;
use futures::sync::mpsc;
use futures::Async;
use futures::Poll;
use futures::Stream;
use httpbis::ServerIncreaseInWindow;
use result;
use server::req_handler::ServerRequestStreamHandler;

pub(crate) enum HandlerToStream<Req: Send + 'static> {
    Message(Req, u32),
    EndStream,
    Error(error::Error),
    BufferProcessed(usize),
}

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
            .map_err(|_| error::Error::Other("xxx"))?)
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

    fn error(&mut self, error: error::Error) -> result::Result<()> {
        self.send(HandlerToStream::Error(error))
    }

    fn buffer_processed(&mut self, buffered: usize) -> result::Result<()> {
        self.send(HandlerToStream::BufferProcessed(buffered))
    }
}

impl<Req: Send + 'static> Stream for ServerRequestStream<Req> {
    type Item = Req;
    type Error = error::Error;

    fn poll(&mut self) -> Poll<Option<Req>, error::Error> {
        loop {
            // TODO: error
            let item = match self.req.poll().map_err(|_| error::Error::Other("xxx"))? {
                Async::Ready(Some(r)) => r,
                Async::Ready(None) => return Err(error::Error::Other("unexpected EOF")),
                Async::NotReady => return Ok(Async::NotReady),
            };

            match item {
                HandlerToStream::Message(req, frame_size) => {
                    // TODO: increase on next poll
                    self.increase_in_window.data_frame_processed(frame_size);
                    self.increase_in_window.increase_window_auto()?;
                    return Ok(Async::Ready(Some(req)));
                }
                HandlerToStream::Error(error) => {
                    return Err(error);
                }
                HandlerToStream::BufferProcessed(buffered) => {
                    // TODO: overflow
                    self.increase_in_window
                        .increase_window_auto_above(buffered as u32)?;
                    continue;
                }
                HandlerToStream::EndStream => {
                    return Ok(Async::Ready(None));
                }
            }
        }
    }
}
