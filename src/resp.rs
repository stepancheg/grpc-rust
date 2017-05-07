use httpbis::futures_misc::stream_single;

use futures::future;
use futures::future::Future;
use futures::stream;
use futures::stream::Stream;

use error::*;
use result::*;
use futures_grpc::*;
use iter::*;
use metadata::*;

/// Single message response
// TODO: trailers metadata
pub struct GrpcSingleResponse<T : Send + 'static>(pub GrpcFutureSend<(GrpcMetadata, GrpcFutureSend<T>)>);

impl<T : Send + 'static> GrpcSingleResponse<T> {
    // constructors

    pub fn metadata_and_future<F>(metadata: GrpcMetadata, result: F) -> GrpcSingleResponse<T>
        where F : Future<Item=T, Error=GrpcError> + Send + 'static
    {
        let boxed: GrpcFutureSend<T> = Box::new(result);
        GrpcSingleResponse(Box::new(future::ok((metadata, boxed))))
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

    // getters

    pub fn join_metadata_result(self) -> GrpcFutureSend<(GrpcMetadata, T)> {
        Box::new(self.0.and_then(|(metadata, result)| result.map(|result| (metadata, result))))
    }

    pub fn drop_metadata(self) -> GrpcFutureSend<T> {
        Box::new(self.0.and_then(|(_metadata, result)| result))
    }

    /// Convert self into single element stream.
    pub fn into_stream(self) -> GrpcStreamingResponse<T> {
        GrpcStreamingResponse(Box::new(self.0.map(|(metadata, future)| {
                let stream: GrpcStreamSend<T> = Box::new(future.into_stream());
                (metadata, stream)
            })))
    }

    pub fn wait(self) -> GrpcResult<(GrpcMetadata, T)> {
        self.join_metadata_result().wait()
    }

    pub fn wait_drop_metadata(self) -> GrpcResult<T> {
        self.wait().map(|(_metadata, r)| r)
    }
}


/// Streaming response
// TODO: trailers metadata
pub struct GrpcStreamingResponse<T : Send + 'static>(pub GrpcFutureSend<(GrpcMetadata, GrpcStreamSend<T>)>);

impl<T : Send + 'static> GrpcStreamingResponse<T> {
    // constructors

    pub fn new<F>(f: F) -> GrpcStreamingResponse<T>
        where F : Future<Item=(GrpcMetadata, GrpcStreamSend<T>), Error=GrpcError> + Send + 'static
    {
        GrpcStreamingResponse(Box::new(f))
    }

    pub fn metadata_and_stream<S>(metadata: GrpcMetadata, result: S) -> GrpcStreamingResponse<T>
        where S : Stream<Item=T, Error=GrpcError> + Send + 'static
    {
        let boxed: GrpcStreamSend<T> = Box::new(result);
        GrpcStreamingResponse::new(future::ok((metadata, boxed)))
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

    pub fn no_metadata<F>(r: F) -> GrpcStreamingResponse<T>
        where F : Stream<Item=T, Error=GrpcError> + Send + 'static
    {
        GrpcStreamingResponse::metadata_and_stream(GrpcMetadata::new(), r)
    }

    // getters

    fn map_stream<U, F>(self, f: F) -> GrpcStreamingResponse<U>
        where
            U : Send + 'static,
            F : FnOnce(GrpcStreamSend<T>) -> GrpcStreamSend<U> + Send + 'static,
    {
        GrpcStreamingResponse(Box::new(self.0.map(move |(metadata, stream)| {
            (metadata, f(stream))
        })))
    }

    pub fn map_items<U, F>(self, f: F) -> GrpcStreamingResponse<U>
        where
            U : Send + 'static,
            F : FnMut(T) -> U + Send + 'static,
    {
        self.map_stream(move |stream| Box::new(stream.map(f)))
    }

    pub fn and_then_items<U, F>(self, f: F) -> GrpcStreamingResponse<U>
        where
            U : Send + 'static,
            F : FnMut(T) -> GrpcResult<U> + Send + 'static,
    {
        self.map_stream(move |stream| Box::new(stream.and_then(f)))
    }

    pub fn drop_metadata(self) -> GrpcStreamSend<T> {
        Box::new(self.0.map(|(_metadata, stream)| stream).flatten_stream())
    }

    pub fn into_future(self) -> GrpcSingleResponse<Vec<T>> {
        GrpcSingleResponse(
            Box::new(self.0.map(|(metadata, stream)| {
                let future: GrpcFutureSend<Vec<T>> = Box::new(stream.collect());
                (metadata, future)
            })))
    }

    /// Take single element from stream
    pub fn single(self) -> GrpcSingleResponse<T> {
        GrpcSingleResponse(
            Box::new(self.0.map(|(metadata, stream)| {
                let future: GrpcFutureSend<T> = Box::new(stream_single(stream));
                (metadata, future)
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
