use futures::Future;
use futures::BoxFuture;
use futures::stream::Stream;
use futures::stream::BoxStream;

use error::Error;


pub type GrpcFuture<T> = Box<Future<Item=T, Error=Error>>;
pub type GrpcStream<T> = Box<Stream<Item=T, Error=Error>>;

pub type GrpcFutureSend<T> = BoxFuture<T, Error>;
pub type GrpcStreamSend<T> = BoxStream<T, Error>;

