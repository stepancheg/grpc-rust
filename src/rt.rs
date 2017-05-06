//! Functions used by generated code.

use futures;
use futures::stream;
use futures::stream::Stream;

use futures_cpupool::CpuPool;

use futures_grpc::*;


use error::*;
use result::*;
use iter::*;
use resp::*;


pub fn stream_to_iter<T : Send + 'static>(s: GrpcStreamingResponse<T>) -> GrpcIterator<T> {
    s.wait_drop_metadata()
}

pub fn iter_to_stream<T : Send + 'static>(i: GrpcIterator<T>) -> GrpcStreamSend<T> {
    Box::new(stream::iter(i))
}

pub fn sync_to_async_unary<Req, Resp, H>(cpupool: &CpuPool, req: Req, sync_handler: H) -> GrpcSingleResponse<Resp>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
        H : FnOnce(Req) -> GrpcResult<Resp> + Send + 'static,
{
    GrpcSingleResponse::no_metadata(cpupool
        .spawn(futures::lazy(move || {
            sync_handler(req)
        })))
}

pub fn sync_to_async_client_streaming<Req, Resp, H>(cpupool: &CpuPool, req: GrpcStreamSend<Req>, sync_handler: H) -> GrpcSingleResponse<Resp>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
        H : FnOnce(GrpcIterator<Req>) -> GrpcResult<Resp> + Send + 'static,
{
    GrpcSingleResponse::no_metadata(cpupool
        .spawn(futures::lazy(move || {
            sync_handler(Box::new(req.wait()))
        })))
}

pub fn sync_to_async_server_streaming<Req, Resp, H>(cpupool: &CpuPool, req: Req, sync_handler: H) -> GrpcStreamingResponse<Resp>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
        H : FnOnce(Req) -> GrpcIterator<Resp> + Send + 'static,
{
    let (sender, receiver) = futures::sync::mpsc::unbounded();
    drop(cpupool
        .spawn(futures::lazy(move || {
            for result in sync_handler(req) {
                // TODO: do not unwrap
                sender.send(result).expect("failed to send");
            }
            futures::finished::<_, GrpcError>(())
        })));
        // TODO: handle cpupool error

    let receiver = receiver.then(|r| {
        match r {
            Ok(r) => r,
            Err(()) => Err(GrpcError::Other("receive")),
        }
    });

    GrpcStreamingResponse::no_metadata(receiver)
}

pub fn sync_to_async_bidi<Req, Resp, H>(cpupool: &CpuPool, req: GrpcStreamSend<Req>, sync_handler: H) -> GrpcStreamingResponse<Resp>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
        H : FnOnce(GrpcIterator<Req>) -> GrpcIterator<Resp> + Send + 'static,
{
    let (sender, receiver) = futures::sync::mpsc::unbounded();
    drop(cpupool
        .spawn(futures::lazy(move || {
            for result in sync_handler(Box::new(req.wait())) {
                // TODO: do not unwrap
                sender.send(result).expect("failed to send");
            }
            futures::finished::<_, GrpcError>(())
        })));
        // TODO: handle cpupool error

    let receiver = receiver.then(|r| {
        match r {
            Ok(r) => r,
            Err(()) => Err(GrpcError::Other("receive")),
        }
    });

    GrpcStreamingResponse::no_metadata(receiver)
}
