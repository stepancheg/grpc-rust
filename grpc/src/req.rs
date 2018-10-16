use futures::stream;
use futures::stream::Stream;

use metadata::Metadata;

use error::Error;
use futures_grpc::GrpcStream;

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
