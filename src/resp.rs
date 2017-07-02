use futures::future;
use futures::future::Future;
use futures::stream;
use futures::stream::Stream;

use error;
use result;
use futures_grpc::*;
use iter::*;
use metadata::*;
use stream_item::*;

/// Single message response
pub struct SingleResponse<T : Send + 'static>(
    pub GrpcFuture<(Metadata, GrpcFuture<(T, Metadata)>)>
);

impl<T : Send + 'static> SingleResponse<T> {
    // constructors

    pub fn new<F>(f: F) -> SingleResponse<T>
        where F : Future<Item=(Metadata, GrpcFuture<(T, Metadata)>), Error=error::Error> + Send + 'static
    {
        SingleResponse(Box::new(f))
    }

    pub fn metadata_and_future<F>(metadata: Metadata, result: F) -> SingleResponse<T>
        where F : Future<Item=T, Error=error::Error> + Send + 'static
    {
        SingleResponse::metadata_and_future_and_trailing_metadata(
            metadata,
            result,
            future::ok(Metadata::new()))
    }

    pub fn metadata_and_future_and_trailing_metadata<F, M>(
        metadata: Metadata, result: F, trailing: M) -> SingleResponse<T>
        where
            F : Future<Item=T, Error=error::Error> + Send + 'static,
            M : Future<Item=Metadata, Error=error::Error> + Send + 'static,
    {
        let future: GrpcFuture<(T, Metadata)> = Box::new(result.join(trailing));
        SingleResponse::new(future::finished((metadata, future)))
    }

    pub fn completed_with_metadata_and_trailing_metadata(metadata: Metadata, r: T, trailing:Metadata) -> SingleResponse<T> {
        SingleResponse::metadata_and_future_and_trailing_metadata(metadata, future::ok(r), future::ok(trailing))
    }

    pub fn completed_with_metadata(metadata: Metadata, r: T) -> SingleResponse<T> {
        SingleResponse::completed_with_metadata_and_trailing_metadata(metadata, r, Metadata::new())
    }

    pub fn completed(r: T) -> SingleResponse<T> {
        SingleResponse::completed_with_metadata_and_trailing_metadata(Metadata::new(), r, Metadata::new())
    }

    pub fn no_metadata<F>(r: F) -> SingleResponse<T>
        where F : Future<Item=T, Error=error::Error> + Send + 'static
    {
        SingleResponse::metadata_and_future(Metadata::new(), r)
    }

    pub fn err(err: error::Error) -> SingleResponse<T> {
        SingleResponse::new(future::err(err))
    }

    // getters

    pub fn join_metadata_result(self) -> GrpcFuture<(Metadata, T, Metadata)> {
        Box::new(self.0.and_then(|(initial, result)| {
            result.map(|(result, trailing)| (initial, result, trailing))
        }))
    }

    pub fn drop_metadata(self) -> GrpcFuture<T> {
        Box::new(self.0.and_then(|(_initial, result)| result.map(|(result, _trailing)| result)))
    }

    /// Convert self into single element stream.
    pub fn into_stream(self) -> StreamingResponse<T> {
        StreamingResponse::new(self.0.map(|(metadata, future)| {
            let stream = future.map(|(result, trailing)| {
                stream::iter(vec![
                    Ok(ItemOrMetadata::Item(result)),
                    Ok(ItemOrMetadata::TrailingMetadata(trailing)),
                ])
            }).flatten_stream();

            (metadata, GrpcStreamWithTrailingMetadata::new(stream))
        }))
    }

    pub fn wait(self) -> result::Result<(Metadata, T, Metadata)> {
        self.join_metadata_result().wait()
    }

    pub fn wait_drop_metadata(self) -> result::Result<T> {
        self.wait().map(|(_initial, r, _trailing)| r)
    }
}


/// Streaming response
pub struct StreamingResponse<T : Send + 'static>(
    /// Initial metadata, stream of items followed by trailing metadata
    pub GrpcFuture<(Metadata, GrpcStreamWithTrailingMetadata<T>)>
);

impl<T : Send + 'static> StreamingResponse<T> {
    // constructors

    pub fn new<F>(f: F) -> StreamingResponse<T>
        where F : Future<Item=(Metadata, GrpcStreamWithTrailingMetadata<T>), Error=error::Error> + Send + 'static
    {
        StreamingResponse(Box::new(f))
    }

    pub fn metadata_and_stream_and_trailing_metadata<S, M>(
        metadata: Metadata, result: S, trailing: M) -> StreamingResponse<T>
        where
            S : Stream<Item=T, Error=error::Error> + Send + 'static,
            M : Future<Item=Metadata, Error=error::Error> + Send + 'static
    {
        let boxed = GrpcStreamWithTrailingMetadata::stream_with_trailing_metadata(result, trailing);
        StreamingResponse::new(future::ok((metadata, boxed)))
    }

    pub fn metadata_and_stream<S>(metadata: Metadata, result: S) -> StreamingResponse<T>
        where S : Stream<Item=T, Error=error::Error> + Send + 'static
    {
        let boxed = GrpcStreamWithTrailingMetadata::stream(result);
        StreamingResponse::new(future::ok((metadata, boxed)))
    }

    pub fn no_metadata<S>(s: S) -> StreamingResponse<T>
        where S : Stream<Item=T, Error=error::Error> + Send + 'static
    {
        StreamingResponse::metadata_and_stream_and_trailing_metadata(Metadata::new(), s, future::ok(Metadata::new()))
    }

    pub fn completed_with_metadata_and_trailing_metadata(metadata: Metadata, r: Vec<T>, trailing: Metadata) -> StreamingResponse<T> {
        StreamingResponse::iter_with_metadata_and_trailing_metadata(metadata, r.into_iter(), future::ok(trailing))
    }

    pub fn completed_with_metadata(metadata: Metadata, r: Vec<T>) -> StreamingResponse<T> {
        StreamingResponse::completed_with_metadata_and_trailing_metadata(metadata, r, Metadata::new())
    }

    pub fn iter_with_metadata_and_trailing_metadata<I, M>(metadata: Metadata, iter: I, trailing: M) -> StreamingResponse<T>
        where
            I : Iterator<Item=T> + Send + 'static,
            M : Future<Item=Metadata, Error=error::Error> + Send + 'static
    {
        StreamingResponse::metadata_and_stream_and_trailing_metadata(
            metadata,
            stream::iter(iter.map(Ok)),
            trailing
        )
    }

    pub fn iter_with_metadata<I>(metadata: Metadata, iter: I) -> StreamingResponse<T>
        where I : Iterator<Item=T> + Send + 'static,
    {
        StreamingResponse::iter_with_metadata_and_trailing_metadata(
            metadata,
            iter,
            future::ok(Metadata::new())
        )
    }

    pub fn completed(r: Vec<T>) -> StreamingResponse<T> {
        StreamingResponse::completed_with_metadata_and_trailing_metadata(Metadata::new(), r, Metadata::new())
    }

    pub fn iter<I>(iter: I) -> StreamingResponse<T>
        where I : Iterator<Item=T> + Send + 'static
    {
        StreamingResponse::iter_with_metadata_and_trailing_metadata(Metadata::new(), iter, future::ok(Metadata::new()))
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
            U : Send + 'static,
            F : FnOnce(GrpcStreamWithTrailingMetadata<T>) -> GrpcStreamWithTrailingMetadata<U> + Send + 'static,
    {
        StreamingResponse::new(self.0.map(move |(metadata, stream)| {
            (metadata, f(stream))
        }))
    }

    pub fn map_items<U, F>(self, f: F) -> StreamingResponse<U>
        where
            U : Send + 'static,
            F : FnMut(T) -> U + Send + 'static,
    {
        self.map_stream(move |stream| stream.map_items(f))
    }

    pub fn and_then_items<U, F>(self, f: F) -> StreamingResponse<U>
        where
            U : Send + 'static,
            F : FnMut(T) -> result::Result<U> + Send + 'static,
    {
        self.map_stream(move |stream| stream.and_then_items(f))
    }

    pub fn drop_metadata(self) -> GrpcStream<T> {
        Box::new(self.0.map(|(_metadata, stream)| stream.drop_metadata()).flatten_stream())
    }

    pub fn into_future(self) -> SingleResponse<Vec<T>> {
        SingleResponse::new(self.0.map(|(initial, stream)| {
            let future: GrpcFuture<(Vec<T>, Metadata)> = stream.collect_with_metadata();
            (initial, future)
        }))
    }

    /// Take single element from stream
    pub fn single(self) -> SingleResponse<T> {
        SingleResponse(Box::new(self.0.map(|(metadata, stream)| {
            (metadata, stream.single_with_metadata())
        })))
    }

    pub fn wait(self) -> result::Result<(Metadata, GrpcIterator<T>)> {
        let (metadata, stream) = self.0.wait()?;
        Ok((metadata, Box::new(stream.wait())))
    }

    pub fn wait_drop_metadata(self) -> GrpcIterator<T> {
        Box::new(self.drop_metadata().wait())
    }

    pub fn collect(self) -> GrpcFuture<(Metadata, Vec<T>, Metadata)> {
        self.into_future().join_metadata_result()
    }
}
