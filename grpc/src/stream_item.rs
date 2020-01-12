use futures::future;
use futures::future::FutureExt;
use futures::future::TryFutureExt;
use futures::stream;
use futures::stream::Stream;
use futures::stream::StreamExt;
use futures::stream::TryStreamExt;

use crate::futures_grpc::*;

use crate::error::Error;

use crate::proto::metadata::Metadata;
use crate::result;
use std::future::Future;

/// Either stream item or trailing metadata.
pub enum ItemOrMetadata<T: Send + 'static> {
    Item(T),
    TrailingMetadata(Metadata), // must be the last item in the stream
}

/// Sequence of items with optional trailing metadata
/// Useful as part of GrpcStreamingResult
pub struct GrpcStreamWithTrailingMetadata<T: Send + 'static>(
    /// Stream of items followed by optional trailing metadata
    pub GrpcStream<ItemOrMetadata<T>>,
);

impl<T: Send + 'static> GrpcStreamWithTrailingMetadata<T> {
    // constructors

    /// Box a stream
    pub fn new<S>(stream: S) -> GrpcStreamWithTrailingMetadata<T>
    where
        S: Stream<Item = crate::Result<ItemOrMetadata<T>>> + Send + 'static,
    {
        GrpcStreamWithTrailingMetadata(Box::pin(stream))
    }

    /// Stream of items with no trailing metadata
    pub fn stream<S>(stream: S) -> GrpcStreamWithTrailingMetadata<T>
    where
        S: Stream<Item = crate::Result<T>> + Send + 'static,
    {
        GrpcStreamWithTrailingMetadata::new(stream.map_ok(ItemOrMetadata::Item))
    }

    /// Stream of items with no trailing metadata
    pub fn stream_with_trailing_metadata<S, F>(
        stream: S,
        trailing: F,
    ) -> GrpcStreamWithTrailingMetadata<T>
    where
        S: Stream<Item = crate::Result<T>> + Send + 'static,
        F: Future<Output = crate::Result<Metadata>> + Send + 'static,
    {
        let stream = stream.map_ok(ItemOrMetadata::Item);
        let trailing = trailing
            .map_ok(ItemOrMetadata::TrailingMetadata)
            .into_stream();
        GrpcStreamWithTrailingMetadata::new(stream.chain(trailing))
    }

    /// Single element stream with no trailing metadata
    pub fn once(item: T) -> GrpcStreamWithTrailingMetadata<T> {
        GrpcStreamWithTrailingMetadata::stream(stream::once(future::ok(item)))
    }

    /// Single element stream with trailing metadata
    pub fn once_with_trailing_metadata(
        item: T,
        metadata: Metadata,
    ) -> GrpcStreamWithTrailingMetadata<T> {
        GrpcStreamWithTrailingMetadata::new(
            stream::iter(
                vec![
                    ItemOrMetadata::Item(item),
                    ItemOrMetadata::TrailingMetadata(metadata),
                ]
                .into_iter(),
            )
            .map(Ok),
        )
    }

    /// Create a result from iterator, no metadata
    pub fn iter<I>(iter: I) -> GrpcStreamWithTrailingMetadata<T>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: Send + 'static,
    {
        GrpcStreamWithTrailingMetadata::stream(stream::iter(iter.into_iter()).map(Ok))
    }

    /// Create an empty stream
    pub fn empty() -> GrpcStreamWithTrailingMetadata<T> {
        GrpcStreamWithTrailingMetadata::new(stream::empty())
    }

    /// Create an error
    pub fn err(err: Error) -> GrpcStreamWithTrailingMetadata<T> {
        GrpcStreamWithTrailingMetadata::new(stream::once(future::err(err)))
    }

    // getters

    /// Apply a function to transform item stream
    fn map_stream<U, F>(self, f: F) -> GrpcStreamWithTrailingMetadata<U>
    where
        U: Send + 'static,
        F: FnOnce(GrpcStream<ItemOrMetadata<T>>) -> GrpcStream<ItemOrMetadata<U>> + Send + 'static,
    {
        GrpcStreamWithTrailingMetadata::new(Box::new(f(self.0)))
    }

    /// Transform items of the stream, keep metadata
    pub fn map_items<U, F>(self, mut f: F) -> GrpcStreamWithTrailingMetadata<U>
    where
        U: Send + 'static,
        F: FnMut(T) -> U + Send + 'static,
    {
        self.map_stream(move |stream| {
            Box::pin(stream.map_ok(move |item| match item {
                ItemOrMetadata::Item(i) => ItemOrMetadata::Item(f(i)),
                ItemOrMetadata::TrailingMetadata(m) => ItemOrMetadata::TrailingMetadata(m),
            }))
        })
    }

    /// Apply `and_then` operation to items, preserving trailing metadata
    pub fn and_then_items<U, F>(self, mut f: F) -> GrpcStreamWithTrailingMetadata<U>
    where
        U: Send + 'static,
        F: FnMut(T) -> result::Result<U> + Send + 'static,
    {
        self.map_stream(move |stream| {
            Box::pin(stream.and_then(move |item| {
                future::ready(match item {
                    ItemOrMetadata::Item(i) => f(i).map(ItemOrMetadata::Item),
                    ItemOrMetadata::TrailingMetadata(m) => Ok(ItemOrMetadata::TrailingMetadata(m)),
                })
            }))
        })
    }

    /// Apply `then` operation to items, preserving trailing metadata
    pub fn then_items<F>(self, mut f: F) -> GrpcStreamWithTrailingMetadata<T>
    where
        T: Send + 'static,
        F: FnMut(Result<T, Error>) -> Result<T, Error> + Send + 'static,
    {
        GrpcStreamWithTrailingMetadata::new(self.0.then(move |result| {
            future::ready(match result {
                Ok(item) => match item {
                    ItemOrMetadata::Item(i) => match f(Ok(i)) {
                        Ok(returned_item) => Ok(ItemOrMetadata::Item(returned_item)),
                        Err(e) => Err(e),
                    },
                    ItemOrMetadata::TrailingMetadata(m) => Ok(ItemOrMetadata::TrailingMetadata(m)),
                },
                Err(t) => match f(Err(t)) {
                    Ok(returned_item) => Ok(ItemOrMetadata::Item(returned_item)),
                    Err(e) => Err(e),
                },
            })
        }))
    }

    /// Return raw futures `Stream` without trailing metadata
    pub fn drop_metadata(self) -> GrpcStream<T> {
        Box::pin(self.0.try_filter_map(|item| {
            future::ok(match item {
                ItemOrMetadata::Item(t) => Some(t),
                ItemOrMetadata::TrailingMetadata(_) => None,
            })
        }))
    }

    /// Collect all elements to a stream
    pub fn collect(self) -> GrpcFuture<Vec<T>> {
        Box::pin(self.drop_metadata().try_collect())
    }

    /// Collect all elements and trailing metadata
    pub fn collect_with_metadata(self) -> GrpcFuture<(Vec<T>, Metadata)> {
        Box::pin(self.0.try_fold(
            (Vec::new(), Metadata::new()),
            |(mut vec, mut metadata), item| {
                // Could also check that elements returned in proper order
                match item {
                    ItemOrMetadata::Item(r) => vec.push(r),
                    ItemOrMetadata::TrailingMetadata(next) => metadata.extend(next),
                }
                future::ok::<_, Error>((vec, metadata))
            },
        ))
    }

    /// Collect stream into single element future.
    /// It is an error if stream has no result items or more than one result item.
    pub fn single_with_metadata(self) -> GrpcFuture<(T, Metadata)> {
        Box::pin(self.collect_with_metadata().and_then(|(mut v, trailing)| {
            future::ready({
                if v.is_empty() {
                    Err(Error::Other("no result"))
                } else if v.len() > 1 {
                    Err(Error::Other("more than one result"))
                } else {
                    Ok((v.swap_remove(0), trailing))
                }
            })
        }))
    }
}
