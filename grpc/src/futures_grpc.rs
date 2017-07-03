use futures::BoxFuture;
use futures::stream::BoxStream;

use error::Error;


pub type GrpcFuture<T> = BoxFuture<T, Error>;
pub type GrpcStream<T> = BoxStream<T, Error>;

