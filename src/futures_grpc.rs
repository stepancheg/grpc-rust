use futures::Future;
use futures::BoxFuture;
use futures::stream::Stream;
use futures::stream::BoxStream;

use error::GrpcError;


pub type GrpcFuture<T> = Box<Future<Item=T, Error=GrpcError>>;
pub type GrpcStream<T> = Box<Stream<Item=T, Error=GrpcError>>;

pub type GrpcFutureSend<T> = BoxFuture<T, GrpcError>;
pub type GrpcStreamSend<T> = BoxStream<T, GrpcError>;

