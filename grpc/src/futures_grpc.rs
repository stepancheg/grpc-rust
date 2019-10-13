use futures::Future;
use futures::Stream;

use error::Error;

pub type GrpcFuture<T> = Box<dyn Future<Item = T, Error = Error> + Send + 'static>;
pub type GrpcStream<T> = Box<dyn Stream<Item = T, Error = Error> + Send + 'static>;
