//! Functions used by generated code.

use futures::Future;
use futures::stream;
use futures::stream::Stream;

use futures_cpupool::CpuPool;

use futures_grpc::*;


use error::*;
use result::*;
use iter::*;


pub fn stream_to_iter<T : 'static>(s: GrpcStream<T>) -> GrpcIterator<T> {
    Box::new(s.wait())
}

pub fn iter_to_stream<T : Send + 'static>(i: GrpcIterator<T>) -> GrpcStream<T> {
    stream::iter(i).boxed()
}

pub fn sync_to_async_unary<Resp, H>(cpupool: &CpuPool, sync_handler: H) -> GrpcFuture<Resp>
    where
        Resp : Send + 'static,
        H : FnOnce() -> GrpcResult<Resp> + Send + 'static,
{
    cpupool.execute(sync_handler)
        .map_err(|_| GrpcError::Other("cpupool"))
        .and_then(|r| r)
        .boxed()
}
