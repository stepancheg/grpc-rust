use futures::future;
use futures::stream;

use futures::stream::Stream;
use futures::stream::StreamExt;

use crate::error::Error;
use futures::channel::mpsc;

use crate::futures_grpc::GrpcStream;
use crate::proto::metadata::Metadata;

/// gRPC request options.
#[derive(Debug, Default, Clone)]
pub struct RequestOptions {
    /// Request metadata.
    pub metadata: Metadata,
    /// Request is cacheable (not implemented).
    pub cachable: bool,
}

impl RequestOptions {
    /// Default request options.
    pub fn new() -> RequestOptions {
        Default::default()
    }
}

/// Excluding initial metadata which is passed separately
pub struct StreamingRequest<T: Send + 'static>(pub GrpcStream<T>);

impl<T: Send + 'static> StreamingRequest<T> {
    // constructors

    pub fn new<S>(stream: S) -> StreamingRequest<T>
    where
        S: Stream<Item = crate::Result<T>> + Send + 'static,
    {
        StreamingRequest(Box::pin(stream))
    }

    pub fn once(item: T) -> StreamingRequest<T> {
        StreamingRequest::new(stream::once(future::ok(item)))
    }

    pub fn iter<I>(iter: I) -> StreamingRequest<T>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: Send + 'static,
    {
        StreamingRequest::new(stream::iter(iter.into_iter()).map(Ok))
    }

    pub fn mpsc() -> (StreamingRequestSender<T>, StreamingRequest<T>) {
        let (tx, rx) = mpsc::channel(0);
        let tx = StreamingRequestSender { sender: Some(tx) };
        // TODO: handle drop of sender
        //let rx = StreamingRequest::new(rx.map_err(|_: mpsc::TryRecvError| error::Error::Other("sender died")));
        let rx = StreamingRequest::new(rx.map(Ok));
        (tx, rx)
    }

    pub fn single(item: T) -> StreamingRequest<T> {
        StreamingRequest::new(stream::once(future::ok(item)))
    }

    pub fn empty() -> StreamingRequest<T> {
        StreamingRequest::new(stream::empty())
    }

    pub fn err(err: Error) -> StreamingRequest<T> {
        StreamingRequest::new(stream::once(future::err(err)))
    }
}

pub struct StreamingRequestSender<T: Send + 'static> {
    sender: Option<mpsc::Sender<T>>,
}

/*
impl<T: Send + 'static> Sink<T> for StreamingRequestSender<T> {
    type Error = crate::Error;

    fn start_send(&mut self, item: T) -> StartSend<T, error::Error> {
        match self.sender.as_mut() {
            // TODO: error
            Some(sender) => sender
                .start_send(item)
                .map_err(|_send_error| error::Error::Other("channel closed")),
            None => Err(error::Error::Other("sender closed")),
        }
    }

    fn poll_complete(&mut self) -> Poll<(), error::Error> {
        match self.sender.as_mut() {
            Some(sender) => sender
                .poll_complete()
                .map_err(|_send_error| error::Error::Other("channel closed")),
            None => Err(error::Error::Other("sender closed")),
        }
    }

    fn close(&mut self) -> Poll<(), error::Error> {
        // Close of sender doesn't end receiver stream
        self.sender.take();
        Poll::Ready(Ok(()))
    }
}
*/

impl<T: Send + 'static> Drop for StreamingRequestSender<T> {
    fn drop(&mut self) {
        // TODO: cancel
    }
}
