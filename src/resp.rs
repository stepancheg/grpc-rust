use futures::future;
use futures::future::Future;
use futures::stream;
use futures::stream::Stream;

use error::*;
use result::*;
use futures_grpc::*;
use iter::*;
use metadata::*;
use stream_item::*;

/// Single message response
pub struct GrpcSingleResponse<T : Send + 'static>(
    pub GrpcFutureSend<(GrpcMetadata, GrpcFutureSend<(T, GrpcMetadata)>)>
);

impl<T : Send + 'static> GrpcSingleResponse<T> {
    // constructors

    pub fn new<F>(f: F) -> GrpcSingleResponse<T>
        where F : Future<Item=(GrpcMetadata, GrpcFutureSend<(T, GrpcMetadata)>), Error=GrpcError> + Send + 'static
    {
        GrpcSingleResponse(Box::new(f))
    }

    pub fn metadata_and_future<F>(metadata: GrpcMetadata, result: F) -> GrpcSingleResponse<T>
        where F : Future<Item=T, Error=GrpcError> + Send + 'static
    {
        let boxed: GrpcFutureSend<(T, GrpcMetadata)> = Box::new((result.map(|r| (r, GrpcMetadata::new()))));
        GrpcSingleResponse::new(future::ok((metadata, boxed)))
    }

    pub fn completed_with_metadata(metadata: GrpcMetadata, r: T) -> GrpcSingleResponse<T> {
        GrpcSingleResponse::metadata_and_future(metadata, future::ok(r))
    }

    pub fn completed(r: T) -> GrpcSingleResponse<T> {
        GrpcSingleResponse::completed_with_metadata(GrpcMetadata::new(), r)
    }

    pub fn no_metadata<F>(r: F) -> GrpcSingleResponse<T>
        where F : Future<Item=T, Error=GrpcError> + Send + 'static
    {
        GrpcSingleResponse::metadata_and_future(GrpcMetadata::new(), r)
    }

    pub fn err(err: GrpcError) -> GrpcSingleResponse<T> {
        GrpcSingleResponse::new(future::err(err))
    }

    // getters

    pub fn join_metadata_result(self) -> GrpcFutureSend<(GrpcMetadata, T, GrpcMetadata)> {
        Box::new(self.0.and_then(|(initial, result)| {
            result.map(|(result, trailing)| (initial, result, trailing))
        }))
    }

    pub fn drop_metadata(self) -> GrpcFutureSend<T> {
        Box::new(self.0.and_then(|(_initial, result)| result.map(|(result, _trailing)| result)))
    }

    /// Convert self into single element stream.
    pub fn into_stream(self) -> GrpcStreamingResponse<T> {
        GrpcStreamingResponse::new(self.0.map(|(metadata, future)| {
            let stream = future.map(|(result, trailing)| {
                stream::iter(vec![
                    Ok(GrpcItemOrMetadata::Item(result)),
                    Ok(GrpcItemOrMetadata::TrailingMetadata(trailing)),
                ])
            }).flatten_stream();

            (metadata, GrpcStreamWithTrailingMetadata::new(stream))
        }))
    }

    pub fn wait(self) -> GrpcResult<(GrpcMetadata, T, GrpcMetadata)> {
        self.join_metadata_result().wait()
    }

    pub fn wait_drop_metadata(self) -> GrpcResult<T> {
        self.wait().map(|(_initial, r, _trailing)| r)
    }
}


/// Streaming response
pub struct GrpcStreamingResponse<T : Send + 'static>(
    pub GrpcFutureSend<(GrpcMetadata, GrpcStreamWithTrailingMetadata<T>)>
);

impl<T : Send + 'static> GrpcStreamingResponse<T> {
    // constructors

    pub fn new<F>(f: F) -> GrpcStreamingResponse<T>
        where F : Future<Item=(GrpcMetadata, GrpcStreamWithTrailingMetadata<T>), Error=GrpcError> + Send + 'static
    {
        GrpcStreamingResponse(Box::new(f))
    }

    pub fn metadata_and_stream<S>(metadata: GrpcMetadata, result: S) -> GrpcStreamingResponse<T>
        where S : Stream<Item=T, Error=GrpcError> + Send + 'static
    {
        let boxed = GrpcStreamWithTrailingMetadata::new(result.map(GrpcItemOrMetadata::Item));
        GrpcStreamingResponse::new(future::ok((metadata, boxed)))
    }

    pub fn no_metadata<S>(s: S) -> GrpcStreamingResponse<T>
        where S : Stream<Item=T, Error=GrpcError> + Send + 'static
    {
        GrpcStreamingResponse::metadata_and_stream(GrpcMetadata::new(), s)
    }

    pub fn completed_with_metadata(metadata: GrpcMetadata, r: Vec<T>) -> GrpcStreamingResponse<T> {
        GrpcStreamingResponse::iter_with_metadata(metadata, r.into_iter())
    }

    pub fn iter_with_metadata<I>(metadata: GrpcMetadata, iter: I) -> GrpcStreamingResponse<T>
        where I: Iterator<Item=T> + Send + 'static
    {
        GrpcStreamingResponse::metadata_and_stream(metadata, stream::iter(iter.map(Ok)))
    }

    pub fn completed(r: Vec<T>) -> GrpcStreamingResponse<T> {
        GrpcStreamingResponse::completed_with_metadata(GrpcMetadata::new(), r)
    }

    pub fn iter<I>(iter: I) -> GrpcStreamingResponse<T>
        where I : Iterator<Item=T> + Send + 'static
    {
        GrpcStreamingResponse::iter_with_metadata(GrpcMetadata::new(), iter)
    }

    pub fn empty() -> GrpcStreamingResponse<T> {
        GrpcStreamingResponse::no_metadata(stream::empty())
    }

    pub fn err(err: GrpcError) -> GrpcStreamingResponse<T> {
        GrpcStreamingResponse::new(future::err(err))
    }

    // getters

    fn map_stream<U, F>(self, f: F) -> GrpcStreamingResponse<U>
        where
            U : Send + 'static,
            F : FnOnce(GrpcStreamWithTrailingMetadata<T>) -> GrpcStreamWithTrailingMetadata<U> + Send + 'static,
    {
        GrpcStreamingResponse::new(self.0.map(move |(metadata, stream)| {
            (metadata, f(stream))
        }))
    }

    pub fn map_items<U, F>(self, f: F) -> GrpcStreamingResponse<U>
        where
            U : Send + 'static,
            F : FnMut(T) -> U + Send + 'static,
    {
        self.map_stream(move |stream| stream.map_items(f))
    }

    pub fn and_then_items<U, F>(self, f: F) -> GrpcStreamingResponse<U>
        where
            U : Send + 'static,
            F : FnMut(T) -> GrpcResult<U> + Send + 'static,
    {
        self.map_stream(move |stream| stream.and_then_items(f))
    }

    pub fn drop_metadata(self) -> GrpcStreamSend<T> {
        Box::new(self.0.map(|(_metadata, stream)| stream.drop_metadata()).flatten_stream())
    }

    pub fn into_future(self) -> GrpcSingleResponse<Vec<T>> {
        GrpcSingleResponse::new(self.0.map(|(initial, stream)| {
            let future: GrpcFutureSend<(Vec<T>, GrpcMetadata)> = stream.collect_with_metadata();
            (initial, future)
        }))
    }

    /// Take single element from stream
    pub fn single(self) -> GrpcSingleResponse<T> {
        GrpcSingleResponse(Box::new(self.0.map(|(metadata, stream)| {
            (metadata, stream.single_with_metadata())
        })))
    }

    pub fn wait(self) -> GrpcResult<(GrpcMetadata, GrpcIterator<T>)> {
        let (metadata, stream) = self.0.wait()?;
        Ok((metadata, Box::new(stream.wait())))
    }

    pub fn wait_drop_metadata(self) -> GrpcIterator<T> {
        Box::new(self.drop_metadata().wait())
    }
}
