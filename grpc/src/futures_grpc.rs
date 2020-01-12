use futures::Future;
use futures::Stream;
use std::pin::Pin;

pub type GrpcFuture<T> = Pin<Box<dyn Future<Output = crate::Result<T>> + Send + 'static>>;
pub type GrpcStream<T> = Pin<Box<dyn Stream<Item = crate::Result<T>> + Send + 'static>>;
