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

pub fn sync_to_async_unary<Req, Resp, H>(cpupool: &CpuPool, req: Req, sync_handler: H) -> GrpcFuture<Resp>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
        H : FnOnce(Req) -> GrpcResult<Resp> + Send + 'static,
{
    cpupool.execute(move || sync_handler(req))
        .map_err(|_| GrpcError::Other("cpupool"))
        .and_then(|r| r)
        .boxed()
}

pub fn sync_to_async_client_streaming<Req, Resp, H>(cpupool: &CpuPool, req: GrpcStream<Req>, sync_handler: H) -> GrpcFuture<Resp>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
        H : FnOnce(GrpcIterator<Req>) -> GrpcResult<Resp> + Send + 'static,
{
    cpupool.execute(move || sync_handler(Box::new(req.wait())))
        .map_err(|_| GrpcError::Other("cpupool"))
        .and_then(|r| r)
        .boxed()
}

pub fn sync_to_async_server_streaming<Req, Resp, H>(cpupool: &CpuPool, req: Req, sync_handler: H) -> GrpcStream<Resp>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
        H : FnOnce(Req) -> GrpcIterator<Resp> + Send + 'static,
{
    unimplemented!()
}

pub fn sync_to_async_bidi<Req, Resp, H>(cpupool: &CpuPool, req: GrpcStream<Req>, sync_handler: H) -> GrpcStream<Resp>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
        H : FnOnce(GrpcIterator<Req>) -> GrpcIterator<Resp> + Send + 'static,
{
    unimplemented!()
}
