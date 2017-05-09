use futures::stream::Stream;

use metadata::GrpcMetadata;

use futures_grpc::GrpcStreamSend;
use stream_item::*;
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
pub struct GrpcStreamingRequest<T : Send + 'static>(pub GrpcStreamWithTrailingMetadata<T>);

impl<T : Send + 'static> GrpcStreamingRequest<T> {

    // constructors

    pub fn new<S>(stream: S) -> GrpcStreamingRequest<T>
        where S : Stream<Item=GrpcItemOrMetadata<T>, Error=GrpcError> + Send + 'static
    {
        GrpcStreamingRequest(GrpcStreamWithTrailingMetadata::new(stream))
    }

    pub fn stream<S>(stream: S) -> GrpcStreamingRequest<T>
        where S : Stream<Item=T, Error=GrpcError> + Send + 'static
    {
        GrpcStreamingRequest(GrpcStreamWithTrailingMetadata::stream(stream))
    }

    pub fn once(item: T) -> GrpcStreamingRequest<T> {
        GrpcStreamingRequest(GrpcStreamWithTrailingMetadata::once(item))
    }

    pub fn once_with_trailing_metadata(item: T, metadata: GrpcMetadata) -> GrpcStreamingRequest<T> {
        GrpcStreamingRequest(GrpcStreamWithTrailingMetadata::once_with_trailing_metadata(item, metadata))
    }

    pub fn iter<I>(iter: I) -> GrpcStreamingRequest<T>
        where
            I : IntoIterator<Item=T>,
            I::IntoIter : Send + 'static,
    {
        GrpcStreamingRequest(GrpcStreamWithTrailingMetadata::iter(iter))
    }

    pub fn empty() -> GrpcStreamingRequest<T> {
        GrpcStreamingRequest(GrpcStreamWithTrailingMetadata::empty())
    }

    pub fn err(err: GrpcError) -> GrpcStreamingRequest<T> {
        GrpcStreamingRequest(GrpcStreamWithTrailingMetadata::err(err))
    }

    // getters

    pub fn drop_metadata(self) -> GrpcStreamSend<T> {
        self.0.drop_metadata()
    }
}
