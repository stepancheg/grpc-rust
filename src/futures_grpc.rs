use futures::BoxFuture;
use futures::stream::BoxStream;

use error::GrpcError;

pub type GrpcFuture<T> = BoxFuture<T, GrpcError>;
pub type GrpcStream<T> = BoxStream<T, GrpcError>;
