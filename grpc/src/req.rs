use futures::stream;
use futures::stream::Stream;

use error;
use error::Error;
use futures::sync::mpsc;
use futures::Async;
use futures::Poll;
use futures::Sink;
use futures::StartSend;
use futures_grpc::GrpcStream;
use proto::metadata::Metadata;


#[derive(Debug, Default)]
pub struct RequestOptions {
    pub metadata: Metadata,
}

impl RequestOptions {
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
        S: Stream<Item = T, Error = Error> + Send + 'static,
    {
        StreamingRequest(Box::new(stream))
    }

    pub fn once(item: T) -> StreamingRequest<T> {
        StreamingRequest::new(stream::once(Ok(item)))
    }

    pub fn iter<I>(iter: I) -> StreamingRequest<T>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: Send + 'static,
    {
        StreamingRequest::new(stream::iter_ok(iter.into_iter()))
    }

    pub fn mpsc() -> (StreamingRequestSender<T>, StreamingRequest<T>) {
        let (tx, rx) = mpsc::channel(0);
        let tx = StreamingRequestSender { sender: Some(tx) };
        // TODO: handle drop of sender
        let rx = StreamingRequest::new(rx.map_err(|()| error::Error::Other("sender died")));
        (tx, rx)
    }

    pub fn single(item: T) -> StreamingRequest<T> {
        StreamingRequest::new(stream::once(Ok(item)))
    }

    pub fn empty() -> StreamingRequest<T> {
        StreamingRequest::new(stream::empty())
    }

    pub fn err(err: Error) -> StreamingRequest<T> {
        StreamingRequest::new(stream::once(Err(err)))
    }
}

pub struct StreamingRequestSender<T: Send + 'static> {
    sender: Option<mpsc::Sender<T>>,
}

impl<T: Send + 'static> Sink for StreamingRequestSender<T> {
    type SinkItem = T;
    type SinkError = error::Error;

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
        Ok(Async::Ready(()))
    }
}

impl<T: Send + 'static> Drop for StreamingRequestSender<T> {
    fn drop(&mut self) {
        // TODO: cancel
    }
}
