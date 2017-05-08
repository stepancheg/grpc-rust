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

pub enum GrpcStreamingRequestItem<T> {
    Item(T),
    TrailingMetadata(GrpcMetadata),
}

/// Excluding initial metadata which is passed separately
pub struct GrpcStreamingRequest<T : Send + 'static>(pub GrpcStreamSend<GrpcStreamingRequestItem<T>>);

impl<T : Send + 'static> GrpcStreamingRequest<T> {

    // constructors

    pub fn new<S>(stream: S) -> GrpcStreamingRequest<T>
        where S : Stream<Item=GrpcStreamingRequestItem<T>, Error=GrpcError> + Send + 'static
    {
        GrpcStreamingRequest(Box::new(stream))
    }

    pub fn stream<S>(stream: S) -> GrpcStreamingRequest<T>
        where S : Stream<Item=T, Error=GrpcError> + Send + 'static
    {
        GrpcStreamingRequest::new(stream.map(GrpcStreamingRequestItem::Item))
    }

    pub fn once(item: T) -> GrpcStreamingRequest<T> {
        GrpcStreamingRequest::stream(stream::once(Ok(item)))
    }

    pub fn once_with_trailing_metadata(item: T, metadata: GrpcMetadata) -> GrpcStreamingRequest<T> {
        GrpcStreamingRequest::new(stream::iter(vec![
            GrpcStreamingRequestItem::Item(item),
            GrpcStreamingRequestItem::TrailingMetadata(metadata),
        ].into_iter().map(Ok)))
    }

    pub fn iter<I>(iter: I) -> GrpcStreamingRequest<T>
        where
            I : IntoIterator<Item=T>,
            I::IntoIter : Send + 'static,
    {
        GrpcStreamingRequest::stream(stream::iter(iter.into_iter().map(Ok)))
    }

    pub fn empty() -> GrpcStreamingRequest<T> {
        GrpcStreamingRequest::new(stream::empty())
    }

    pub fn err(err: GrpcError) -> GrpcStreamingRequest<T> {
        GrpcStreamingRequest::new(stream::once(Err(err)))
    }

    // getters

    pub fn drop_metadata(self) -> GrpcStreamSend<T> {
        Box::new(self.0.filter_map(|item| {
            match item {
                GrpcStreamingRequestItem::Item(t) => Some(t),
                GrpcStreamingRequestItem::TrailingMetadata(_) => None,
            }
        }))
    }
}
