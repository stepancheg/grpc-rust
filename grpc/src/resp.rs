use futures::future;
use futures::future::FutureExt;
use futures::future::TryFutureExt;
use futures::stream;
use futures::stream::Stream;
use futures::stream::StreamExt;

use crate::error;

use crate::futures_grpc::*;

use crate::proto::metadata::Metadata;
use crate::result;
use crate::stream_item::*;
use futures::task::Context;
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

/// Single message response
pub struct SingleResponse<T: Send + 'static>(pub GrpcFuture<(Metadata, GrpcFuture<(T, Metadata)>)>);

impl<T: Send + 'static> SingleResponse<T> {
    // constructors

    pub fn new<F>(f: F) -> SingleResponse<T>
    where
        F: Future<Output = crate::Result<(Metadata, GrpcFuture<(T, Metadata)>)>> + Send + 'static,
    {
        SingleResponse(Box::pin(f))
    }

    pub fn metadata_and_future<F>(metadata: Metadata, result: F) -> SingleResponse<T>
    where
        F: Future<Output = crate::Result<T>> + Send + 'static,
    {
        SingleResponse::metadata_and_future_and_trailing_metadata(
            metadata,
            result,
            future::ok(Metadata::new()),
        )
    }

    pub fn metadata_and_future_and_trailing_metadata<F, M>(
        metadata: Metadata,
        result: F,
        trailing: M,
    ) -> SingleResponse<T>
    where
        F: Future<Output = crate::Result<T>> + Send + 'static,
        M: Future<Output = crate::Result<Metadata>> + Send + 'static,
    {
        let future: GrpcFuture<(T, Metadata)> = Box::pin(future::try_join(result, trailing));
        SingleResponse::new(future::ok((metadata, future)))
    }

    pub fn completed_with_metadata_and_trailing_metadata(
        metadata: Metadata,
        r: T,
        trailing: Metadata,
    ) -> SingleResponse<T> {
        SingleResponse::metadata_and_future_and_trailing_metadata(
            metadata,
            future::ok(r),
            future::ok(trailing),
        )
    }

    pub fn completed_with_metadata(metadata: Metadata, r: T) -> SingleResponse<T> {
        SingleResponse::completed_with_metadata_and_trailing_metadata(metadata, r, Metadata::new())
    }

    pub fn completed(r: T) -> SingleResponse<T> {
        SingleResponse::completed_with_metadata_and_trailing_metadata(
            Metadata::new(),
            r,
            Metadata::new(),
        )
    }

    pub fn no_metadata<F>(r: F) -> SingleResponse<T>
    where
        F: Future<Output = crate::Result<T>> + Send + 'static,
    {
        SingleResponse::metadata_and_future(Metadata::new(), r)
    }

    pub fn err(err: error::Error) -> SingleResponse<T> {
        SingleResponse::new(future::err(err))
    }

    // getters

    pub fn join_metadata_result(self) -> GrpcFuture<(Metadata, T, Metadata)> {
        Box::pin(self.0.and_then(|(initial, result)| {
            result.map_ok(|(result, trailing)| (initial, result, trailing))
        }))
    }

    pub fn drop_metadata(self) -> GrpcFuture<T> {
        Box::pin(
            self.0
                .and_then(|(_initial, result)| result.map_ok(|(result, _trailing)| result)),
        )
    }

    /// Convert self into single element stream.
    pub fn into_stream(self) -> StreamingResponse<T> {
        StreamingResponse::new(self.0.map_ok(|(metadata, future)| {
            let stream = future
                .map_ok(|(result, trailing)| {
                    stream::iter(vec![
                        ItemOrMetadata::Item(result),
                        ItemOrMetadata::TrailingMetadata(trailing),
                    ])
                    .map(Ok)
                })
                .try_flatten_stream();

            (metadata, GrpcStreamWithTrailingMetadata::new(stream))
        }))
    }
}

impl<T: Send + 'static> Future for SingleResponse<T> {
    type Output = crate::Result<(Metadata, GrpcFuture<(T, Metadata)>)>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<crate::Result<(Metadata, GrpcFuture<(T, Metadata)>)>> {
        self.0.poll_unpin(cx)
    }
}

/// Streaming response
pub struct StreamingResponse<T: Send + 'static>(
    /// Initial metadata, stream of items followed by trailing metadata
    pub GrpcFuture<(Metadata, GrpcStreamWithTrailingMetadata<T>)>,
);

fn _assert_types() {
    //    ::assert_types::assert_send::<StreamingResponse<String>>();
    //    ::assert_types::assert_sync::<StreamingResponse<String>>();
}

impl<T: Send + 'static> StreamingResponse<T> {
    // constructors

    pub fn new<F>(f: F) -> StreamingResponse<T>
    where
        F: Future<Output = crate::Result<(Metadata, GrpcStreamWithTrailingMetadata<T>)>>
            + Send
            + 'static,
    {
        StreamingResponse(Box::pin(f))
    }

    pub fn metadata_and_stream_and_trailing_metadata<S, M>(
        metadata: Metadata,
        result: S,
        trailing: M,
    ) -> StreamingResponse<T>
    where
        S: Stream<Item = crate::Result<T>> + Send + 'static,
        M: Future<Output = crate::Result<Metadata>> + Send + 'static,
    {
        let boxed = GrpcStreamWithTrailingMetadata::stream_with_trailing_metadata(result, trailing);
        StreamingResponse::new(future::ok((metadata, boxed)))
    }

    pub fn metadata_and_stream<S>(metadata: Metadata, result: S) -> StreamingResponse<T>
    where
        S: Stream<Item = crate::Result<T>> + Send + 'static,
    {
        let boxed = GrpcStreamWithTrailingMetadata::stream(result);
        StreamingResponse::new(future::ok((metadata, boxed)))
    }

    pub fn no_metadata<S>(s: S) -> StreamingResponse<T>
    where
        S: Stream<Item = crate::Result<T>> + Send + 'static,
    {
        StreamingResponse::metadata_and_stream_and_trailing_metadata(
            Metadata::new(),
            s,
            future::ok(Metadata::new()),
        )
    }

    pub fn completed_with_metadata_and_trailing_metadata(
        metadata: Metadata,
        r: Vec<T>,
        trailing: Metadata,
    ) -> StreamingResponse<T> {
        StreamingResponse::iter_with_metadata_and_trailing_metadata(
            metadata,
            r.into_iter(),
            future::ok(trailing),
        )
    }

    pub fn completed_with_metadata(metadata: Metadata, r: Vec<T>) -> StreamingResponse<T> {
        StreamingResponse::completed_with_metadata_and_trailing_metadata(
            metadata,
            r,
            Metadata::new(),
        )
    }

    pub fn iter_with_metadata_and_trailing_metadata<I, M>(
        metadata: Metadata,
        iter: I,
        trailing: M,
    ) -> StreamingResponse<T>
    where
        I: Iterator<Item = T> + Send + 'static,
        M: Future<Output = crate::Result<Metadata>> + Send + 'static,
    {
        StreamingResponse::metadata_and_stream_and_trailing_metadata(
            metadata,
            stream::iter(iter).map(Ok),
            trailing,
        )
    }

    pub fn iter_with_metadata<I>(metadata: Metadata, iter: I) -> StreamingResponse<T>
    where
        I: Iterator<Item = T> + Send + 'static,
    {
        StreamingResponse::iter_with_metadata_and_trailing_metadata(
            metadata,
            iter,
            future::ok(Metadata::new()),
        )
    }

    pub fn completed(r: Vec<T>) -> StreamingResponse<T> {
        StreamingResponse::completed_with_metadata_and_trailing_metadata(
            Metadata::new(),
            r,
            Metadata::new(),
        )
    }

    pub fn iter<I>(iter: I) -> StreamingResponse<T>
    where
        I: Iterator<Item = T> + Send + 'static,
    {
        StreamingResponse::iter_with_metadata_and_trailing_metadata(
            Metadata::new(),
            iter,
            future::ok(Metadata::new()),
        )
    }

    pub fn empty() -> StreamingResponse<T> {
        StreamingResponse::no_metadata(stream::empty())
    }

    /// Create an error response
    pub fn err(err: error::Error) -> StreamingResponse<T> {
        StreamingResponse::new(future::err(err))
    }

    // getters

    fn map_stream<U, F>(self, f: F) -> StreamingResponse<U>
    where
        U: Send + 'static,
        F: FnOnce(GrpcStreamWithTrailingMetadata<T>) -> GrpcStreamWithTrailingMetadata<U>
            + Send
            + 'static,
    {
        StreamingResponse::new(
            self.0
                .map_ok(move |(metadata, stream)| (metadata, f(stream))),
        )
    }

    pub fn map_items<U, F>(self, f: F) -> StreamingResponse<U>
    where
        U: Send + 'static,
        F: FnMut(T) -> U + Send + 'static,
    {
        self.map_stream(move |stream| stream.map_items(f))
    }

    pub fn and_then_items<U, F>(self, f: F) -> StreamingResponse<U>
    where
        U: Send + 'static,
        F: FnMut(T) -> result::Result<U> + Send + 'static,
    {
        self.map_stream(move |stream| stream.and_then_items(f))
    }

    pub fn drop_metadata(self) -> GrpcStream<T> {
        Box::pin(
            self.0
                .map_ok(|(_metadata, stream)| stream.drop_metadata())
                .try_flatten_stream(),
        )
    }

    pub fn into_future(self) -> SingleResponse<Vec<T>> {
        SingleResponse::new(self.0.map_ok(|(initial, stream)| {
            let future: GrpcFuture<(Vec<T>, Metadata)> = stream.collect_with_metadata();
            (initial, future)
        }))
    }

    /// Take single element from stream
    pub fn single(self) -> SingleResponse<T> {
        SingleResponse(Box::pin(self.0.map_ok(|(metadata, stream)| {
            (metadata, stream.single_with_metadata())
        })))
    }

    pub fn collect(self) -> GrpcFuture<(Metadata, Vec<T>, Metadata)> {
        self.into_future().join_metadata_result()
    }
}
