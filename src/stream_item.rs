use futures::future;
use futures::future::Future;
use futures::stream;
use futures::stream::Stream;

use futures_grpc::*;

use error::GrpcError;
use result::GrpcResult;
use iter::GrpcIterator;

use metadata::*;

/// Either stream item or trailing metadata.
pub enum GrpcItemOrMetadata<T : Send + 'static> {
    Item(T),
    TrailingMetadata(GrpcMetadata), // must be the last item in the stream
}

/// Sequence of items with optional trailing metadata
/// Useful as part of GrpcStreamingResult
pub struct GrpcStreamWithTrailingMetadata<T : Send + 'static>(
    /// Stream of items followed by optional trailing metadata
    pub GrpcStreamSend<GrpcItemOrMetadata<T>>
);

impl<T : Send + 'static> GrpcStreamWithTrailingMetadata<T> {

    // constructors

    /// Box a stream
    pub fn new<S>(stream: S) -> GrpcStreamWithTrailingMetadata<T>
        where S : Stream<Item=GrpcItemOrMetadata<T>, Error=GrpcError> + Send + 'static
    {
        GrpcStreamWithTrailingMetadata(Box::new(stream))
    }

    /// Stream of items with no trailing metadata
    pub fn stream<S>(stream: S) -> GrpcStreamWithTrailingMetadata<T>
        where S : Stream<Item=T, Error=GrpcError> + Send + 'static
    {
        GrpcStreamWithTrailingMetadata::new(stream.map(GrpcItemOrMetadata::Item))
    }

    /// Single element stream with no trailing metadata
    pub fn once(item: T) -> GrpcStreamWithTrailingMetadata<T> {
        GrpcStreamWithTrailingMetadata::stream(stream::once(Ok(item)))
    }

    /// Single element stream with trailing metadata
    pub fn once_with_trailing_metadata(item: T, metadata: GrpcMetadata) -> GrpcStreamWithTrailingMetadata<T> {
        GrpcStreamWithTrailingMetadata::new(stream::iter(vec![
            GrpcItemOrMetadata::Item(item),
            GrpcItemOrMetadata::TrailingMetadata(metadata),
        ].into_iter().map(Ok)))
    }

    /// Create a result from iterator, no metadata
    pub fn iter<I>(iter: I) -> GrpcStreamWithTrailingMetadata<T>
        where
            I : IntoIterator<Item=T>,
            I::IntoIter : Send + 'static,
    {
        GrpcStreamWithTrailingMetadata::stream(stream::iter(iter.into_iter().map(Ok)))
    }

    /// Create an empty stream
    pub fn empty() -> GrpcStreamWithTrailingMetadata<T> {
        GrpcStreamWithTrailingMetadata::new(stream::empty())
    }

    /// Create an error
    pub fn err(err: GrpcError) -> GrpcStreamWithTrailingMetadata<T> {
        GrpcStreamWithTrailingMetadata::new(stream::once(Err(err)))
    }

    // getters

    /// Apply a function to transform item stream
    fn map_stream<U, F>(self, f: F) -> GrpcStreamWithTrailingMetadata<U>
        where
            U : Send + 'static,
            F : FnOnce(GrpcStreamSend<GrpcItemOrMetadata<T>>) -> GrpcStreamSend<GrpcItemOrMetadata<U>> + Send + 'static,
    {
        GrpcStreamWithTrailingMetadata::new(Box::new(f(self.0)))
    }

    /// Transform items of the stream, keep metadata
    pub fn map_items<U, F>(self, mut f: F) -> GrpcStreamWithTrailingMetadata<U>
        where
            U : Send + 'static,
            F : FnMut(T) -> U + Send + 'static,
    {
        self.map_stream(move |stream| {
            Box::new(stream.map(move |item| {
                match item {
                    GrpcItemOrMetadata::Item(i) => GrpcItemOrMetadata::Item(f(i)),
                    GrpcItemOrMetadata::TrailingMetadata(m) => GrpcItemOrMetadata::TrailingMetadata(m),
                }
            }))
        })
    }

    /// Apply `and_then` operation to items, preserving trailing metadata
    pub fn and_then_items<U, F>(self, mut f: F) -> GrpcStreamWithTrailingMetadata<U>
        where
            U : Send + 'static,
            F : FnMut(T) -> GrpcResult<U> + Send + 'static,
    {
        self.map_stream(move |stream| {
            Box::new(stream.and_then(move |item| {
                match item {
                    GrpcItemOrMetadata::Item(i) => f(i).map(GrpcItemOrMetadata::Item),
                    GrpcItemOrMetadata::TrailingMetadata(m) => Ok(GrpcItemOrMetadata::TrailingMetadata(m)),
                }
            }))
        })
    }

    /// Return raw futures `Stream` without trailing metadata
    pub fn drop_metadata(self) -> GrpcStreamSend<T> {
        Box::new(self.0.filter_map(|item| {
            match item {
                GrpcItemOrMetadata::Item(t) => Some(t),
                GrpcItemOrMetadata::TrailingMetadata(_) => None,
            }
        }))
    }

    /// Collect all elements to a stream
    pub fn collect(self) -> GrpcFutureSend<Vec<T>> {
        Box::new(self.drop_metadata().collect())
    }

    /// Collect all elements and trailing metadata
    pub fn collect_with_metadata(self) -> GrpcFutureSend<(Vec<T>, GrpcMetadata)> {
        Box::new(self.0.fold((Vec::new(), GrpcMetadata::new()), |(mut vec, mut metadata), item| {
            // Could also check that elements returned in proper order
            match item {
                GrpcItemOrMetadata::Item(r) => vec.push(r),
                GrpcItemOrMetadata::TrailingMetadata(next) => metadata.extend(next),
            }
            future::ok::<_, GrpcError>((vec, metadata))
        }))
    }

    /// Collect stream into single element future.
    /// It is an error if stream has no result items or more than one result item.
    pub fn single_with_metadata(self) -> GrpcFutureSend<(T, GrpcMetadata)> {
        Box::new(self.collect_with_metadata().and_then(|(mut v, trailing)| {
            if v.is_empty() {
                Err(GrpcError::Other("no result"))
            } else if v.len() > 1 {
                Err(GrpcError::Other("more than one result"))
            } else {
                Ok((v.swap_remove(0), trailing))
            }
        }))
    }

    /// Convert `Stream` to synchronous `Iterator`.
    /// Metadata is dropped.
    pub fn wait(self) -> GrpcIterator<T> {
        Box::new(self.drop_metadata().wait())
    }

}
