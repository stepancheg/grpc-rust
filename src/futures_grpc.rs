use futures::future;
use futures::Future;
use futures::BoxFuture;
use futures::stream;
use futures::stream::Stream;
use futures::stream::BoxStream;

use error::GrpcError;
use metadata::Metadata;


pub type GrpcFuture<T> = Box<Future<Item=T, Error=GrpcError>>;
pub type GrpcStream<T> = Box<Stream<Item=T, Error=GrpcError>>;

pub type GrpcFutureSend<T> = BoxFuture<T, GrpcError>;
pub type GrpcStreamSend<T> = BoxStream<T, GrpcError>;

/// Single message request or response
pub struct GrpcCallFuture<T : Send + 'static> {
    // TODO: trailers metadata
    future: GrpcFutureSend<(Metadata, GrpcFutureSend<T>)>,
}

impl<T : Send + 'static> GrpcCallFuture<T> {
    // constructors

    pub fn metadata_and_future<F>(metadata: Metadata, result: F) -> GrpcCallFuture<T>
        where F : Future<Item=T, Error=GrpcError> + Send + 'static
    {
        let boxed: GrpcFutureSend<T> = Box::new(result);
        GrpcCallFuture {
            future: Box::new(future::ok((metadata, boxed)))
        }
    }

    pub fn completed_with_metadata(metadata: Metadata, r: T) -> GrpcCallFuture<T> {
        GrpcCallFuture::metadata_and_future(metadata, future::ok(r))
    }

    pub fn completed(r: T) -> GrpcCallFuture<T> {
        GrpcCallFuture::completed_with_metadata(Metadata::new(), r)
    }

    pub fn no_metadata<F>(r: F) -> GrpcCallFuture<T>
        where F : Future<Item=T, Error=GrpcError> + Send + 'static
    {
        GrpcCallFuture::metadata_and_future(Metadata::new(), r)
    }

    // getters

    pub fn join_metadata_result(self) -> GrpcFutureSend<(Metadata, T)> {
        Box::new(self.future.and_then(|(metadata, result)| result.map(|result| (metadata, result))))
    }

    pub fn drop_metadata(self) -> GrpcFutureSend<T> {
        Box::new(self.future.and_then(|(_metadata, result)| result))
    }

    /// Convert self into single element stream.
    pub fn into_stream(self) -> GrpcCallStream<T> {
        GrpcCallStream {
            future: Box::new(self.future.map(|(metadata, future)| {
                let stream: GrpcStreamSend<T> = Box::new(future.into_stream());
                (metadata, stream)
            }))
        }
    }
}


/// Stream request or response
pub struct GrpcCallStream<T : Send + 'static> {
    // TODO: trailers metadata
    future: GrpcFutureSend<(Metadata, GrpcStreamSend<T>)>,
}

impl<T : Send + 'static> GrpcCallStream<T> {
    // constructors

    pub fn metadata_and_stream<S>(metadata: Metadata, result: S) -> GrpcCallStream<T>
        where S : Stream<Item=T, Error=GrpcError> + Send + 'static
    {
        let boxed: GrpcStreamSend<T> = Box::new(result);
        GrpcCallStream {
            future: Box::new(future::ok((metadata, boxed)))
        }
    }

    pub fn completed_with_metadata(metadata: Metadata, r: Vec<T>) -> GrpcCallStream<T> {
        GrpcCallStream::metadata_and_stream(metadata, stream::iter(r.into_iter().map(Ok)))
    }

    pub fn completed(r: Vec<T>) -> GrpcCallStream<T> {
        GrpcCallStream::completed_with_metadata(Metadata::new(), r)
    }

    pub fn no_metadata<F>(r: F) -> GrpcCallStream<T>
        where F : Stream<Item=T, Error=GrpcError> + Send + 'static
    {
        GrpcCallStream::metadata_and_stream(Metadata::new(), r)
    }

    // getters

    pub fn drop_metadata(self) -> GrpcStreamSend<T> {
        Box::new(self.future.map(|(_metadata, stream)| stream).flatten_stream())
    }

    pub fn into_future(self) -> GrpcCallFuture<Vec<T>> {
        GrpcCallFuture {
            future: Box::new(self.future.map(|(metadata, stream)| {
                let future: GrpcFutureSend<Vec<T>> = Box::new(stream.collect());
                (metadata, future)
            }))
        }
    }

}
