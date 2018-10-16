use std::sync::Arc;

use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;

use bytes::Bytes;

use futures::future::Future;
use futures::stream::Stream;

use error::Error;

use method::*;
use req::*;
use resp::*;

use futures_misc::stream_single;
use misc::any_to_string;

pub trait MethodHandler<Req, Resp>
where
    Req: Send + 'static,
    Resp: Send + 'static,
{
    fn handle(&self, m: RequestOptions, req: StreamingRequest<Req>) -> StreamingResponse<Resp>;
}

pub struct MethodHandlerUnary<F> {
    f: Arc<F>,
}

pub struct MethodHandlerServerStreaming<F> {
    f: Arc<F>,
}

pub struct MethodHandlerClientStreaming<F> {
    f: Arc<F>,
}

pub struct MethodHandlerBidi<F> {
    f: Arc<F>,
}

impl<F> GrpcStreamingFlavor for MethodHandlerUnary<F> {
    type Flavor = GrpcStreamingUnary;

    fn streaming() -> GrpcStreaming {
        GrpcStreaming::Unary
    }
}

impl<F> GrpcStreamingFlavor for MethodHandlerClientStreaming<F> {
    type Flavor = GrpcStreamingClientStreaming;

    fn streaming() -> GrpcStreaming {
        GrpcStreaming::ClientStreaming
    }
}

impl<F> GrpcStreamingFlavor for MethodHandlerServerStreaming<F> {
    type Flavor = GrpcStreamingServerStreaming;

    fn streaming() -> GrpcStreaming {
        GrpcStreaming::ServerStreaming
    }
}

impl<F> GrpcStreamingFlavor for MethodHandlerBidi<F> {
    type Flavor = GrpcStreamingBidi;

    fn streaming() -> GrpcStreaming {
        GrpcStreaming::Bidi
    }
}

impl<F> MethodHandlerUnary<F> {
    pub fn new<Req, Resp>(f: F) -> Self
    where
        Req: Send + 'static,
        Resp: Send + 'static,
        F: Fn(RequestOptions, Req) -> SingleResponse<Resp> + Send + 'static,
    {
        MethodHandlerUnary { f: Arc::new(f) }
    }
}

impl<F> MethodHandlerClientStreaming<F> {
    pub fn new<Req, Resp>(f: F) -> Self
    where
        Req: Send + 'static,
        Resp: Send + 'static,
        F: Fn(RequestOptions, StreamingRequest<Req>) -> SingleResponse<Resp> + Send + 'static,
    {
        MethodHandlerClientStreaming { f: Arc::new(f) }
    }
}

impl<F> MethodHandlerServerStreaming<F> {
    pub fn new<Req, Resp>(f: F) -> Self
    where
        Req: Send + 'static,
        Resp: Send + 'static,
        F: Fn(RequestOptions, Req) -> StreamingResponse<Resp> + Send + 'static,
    {
        MethodHandlerServerStreaming { f: Arc::new(f) }
    }
}

impl<F> MethodHandlerBidi<F> {
    pub fn new<Req, Resp>(f: F) -> Self
    where
        Req: Send + 'static,
        Resp: Send + 'static,
        F: Fn(RequestOptions, StreamingRequest<Req>) -> StreamingResponse<Resp> + Send + 'static,
    {
        MethodHandlerBidi { f: Arc::new(f) }
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerUnary<F>
where
    Req: Send + 'static,
    Resp: Send + 'static,
    F: Fn(RequestOptions, Req) -> SingleResponse<Resp> + Send + Sync + 'static,
{
    fn handle(&self, m: RequestOptions, req: StreamingRequest<Req>) -> StreamingResponse<Resp> {
        let f = self.f.clone();
        SingleResponse::new(stream_single(req.0).and_then(move |req| f(m, req).0)).into_stream()
    }
}

impl<Req: Send + 'static, Resp: Send + 'static, F> MethodHandler<Req, Resp>
    for MethodHandlerClientStreaming<F>
where
    Resp: Send + 'static,
    F: Fn(RequestOptions, StreamingRequest<Req>) -> SingleResponse<Resp> + Send + Sync + 'static,
{
    fn handle(&self, m: RequestOptions, req: StreamingRequest<Req>) -> StreamingResponse<Resp> {
        ((self.f)(m, req)).into_stream()
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerServerStreaming<F>
where
    Req: Send + 'static,
    Resp: Send + 'static,
    F: Fn(RequestOptions, Req) -> StreamingResponse<Resp> + Send + Sync + 'static,
{
    fn handle(&self, o: RequestOptions, req: StreamingRequest<Req>) -> StreamingResponse<Resp> {
        let f = self.f.clone();
        StreamingResponse(Box::new(
            stream_single(req.0).and_then(move |req| f(o, req).0),
        ))
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerBidi<F>
where
    Req: Send + 'static,
    Resp: Send + 'static,
    F: Fn(RequestOptions, StreamingRequest<Req>) -> StreamingResponse<Resp> + Send + Sync + 'static,
{
    fn handle(&self, m: RequestOptions, req: StreamingRequest<Req>) -> StreamingResponse<Resp> {
        (self.f)(m, req)
    }
}

pub(crate) trait MethodHandlerDispatch {
    fn start_request(
        &self,
        m: RequestOptions,
        grpc_frames: StreamingRequest<Bytes>,
    ) -> StreamingResponse<Vec<u8>>;
}

struct MethodHandlerDispatchImpl<Req, Resp> {
    desc: Arc<MethodDescriptor<Req, Resp>>,
    method_handler: Box<MethodHandler<Req, Resp> + Sync + Send>,
}

impl<Req, Resp> MethodHandlerDispatch for MethodHandlerDispatchImpl<Req, Resp>
where
    Req: Send + 'static,
    Resp: Send + 'static,
{
    fn start_request(
        &self,
        o: RequestOptions,
        req_grpc_frames: StreamingRequest<Bytes>,
    ) -> StreamingResponse<Vec<u8>> {
        let desc = self.desc.clone();
        let req = req_grpc_frames
            .0
            .and_then(move |frame| desc.req_marshaller.read(frame));
        let resp = catch_unwind(AssertUnwindSafe(|| {
            self.method_handler.handle(o, StreamingRequest::new(req))
        }));
        match resp {
            Ok(resp) => {
                let desc_copy = self.desc.clone();
                resp.and_then_items(move |resp| desc_copy.resp_marshaller.write(&resp))
            }
            Err(e) => {
                let message = any_to_string(e);
                StreamingResponse::err(Error::Panic(message))
            }
        }
    }
}

pub struct ServerMethod {
    pub(crate) name: String,
    pub(crate) dispatch: Box<MethodHandlerDispatch + Sync + Send>,
}

impl ServerMethod {
    pub fn new<Req, Resp, H>(method: Arc<MethodDescriptor<Req, Resp>>, handler: H) -> ServerMethod
    where
        Req: Send + 'static,
        Resp: Send + 'static,
        H: MethodHandler<Req, Resp> + 'static + Sync + Send,
    {
        ServerMethod {
            name: method.name.clone(),
            dispatch: Box::new(MethodHandlerDispatchImpl {
                desc: method,
                method_handler: Box::new(handler),
            }),
        }
    }
}
