use futures::stream;
use futures::stream::Stream;

use metadata::GrpcMetadata;

use futures_grpc::GrpcStreamSend;
use error::GrpcError;

#[derive(Debug, Default)]
pub struct GrpcRequestOptions {
    pub metadata: GrpcMetadata,
}

impl GrpcRequestOptions {
    pub fn new() -> GrpcRequestOptions {
        Default::default()
    }
}

/// Excluding initial metadata which is passed separately
pub struct GrpcStreamingRequest<T : Send + 'static>(pub GrpcStreamSend<T>);

impl<T : Send + 'static> GrpcStreamingRequest<T> {

    // constructors

    pub fn new<S>(stream: S) -> GrpcStreamingRequest<T>
        where S : Stream<Item=T, Error=GrpcError> + Send + 'static
    {
        GrpcStreamingRequest(Box::new(stream))
    }

    pub fn once(item: T) -> GrpcStreamingRequest<T> {
        GrpcStreamingRequest::new(stream::once(Ok(item)))
    }

    pub fn iter<I>(iter: I) -> GrpcStreamingRequest<T>
        where
            I : IntoIterator<Item=T>,
            I::IntoIter : Send + 'static,
    {
        GrpcStreamingRequest::new(stream::iter(iter.into_iter().map(Ok)))
    }

    pub fn empty() -> GrpcStreamingRequest<T> {
        GrpcStreamingRequest::new(stream::empty())
    }

    pub fn err(err: GrpcError) -> GrpcStreamingRequest<T> {
        GrpcStreamingRequest::new(stream::once(Err(err)))
    }
}
