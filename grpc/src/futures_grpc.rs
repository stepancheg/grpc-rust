use futures::Future;
use futures::Stream;

use error::Error;

pub type GrpcFuture<T> = Box<Future<Item = T, Error = Error> + Send + 'static>;
pub type GrpcStream<T> = Box<Stream<Item = T, Error = Error> + Send + 'static>;
