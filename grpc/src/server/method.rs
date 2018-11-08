use std::sync::Arc;

use common::sink::SinkCommon;
use method::GrpcStreaming;
use method::GrpcStreamingBidi;
use method::GrpcStreamingClientStreaming;
use method::GrpcStreamingFlavor;
use method::GrpcStreamingServerStreaming;
use method::GrpcStreamingUnary;
use method::MethodDescriptor;
use result;
use server::ctx::ServerHandlerContext;
use server::req_handler::ServerRequest;
use server::req_handler::ServerRequestUnaryHandler;
use server::req_handler::ServerRequestUntyped;
use server::resp_sink::ServerResponseSink;
use server::resp_sink_untyped::ServerResponseUntypedSink;
use std::marker;
use ServerResponseUnarySink;
use server::req_single::ServerRequestSingle;

pub trait MethodHandler<Req, Resp>
where
    Req: Send + 'static,
    Resp: Send + 'static,
{
    fn handle(
        &self,
        context: ServerHandlerContext,
        req: ServerRequest<Req>,
        resp: ServerResponseSink<Resp>,
    ) -> result::Result<()>;
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
        F: Fn(ServerHandlerContext, ServerRequestSingle<Req>, ServerResponseUnarySink<Resp>) -> result::Result<()>
            + Send
            + 'static,
    {
        MethodHandlerUnary { f: Arc::new(f) }
    }
}

impl<F> MethodHandlerClientStreaming<F> {
    pub fn new<Req, Resp>(f: F) -> Self
    where
        Req: Send + 'static,
        Resp: Send + 'static,
        F: Fn(ServerHandlerContext, ServerRequest<Req>, ServerResponseUnarySink<Resp>)
                -> result::Result<()>
            + Send
            + 'static,
    {
        MethodHandlerClientStreaming { f: Arc::new(f) }
    }
}

impl<F> MethodHandlerServerStreaming<F> {
    pub fn new<Req, Resp>(f: F) -> Self
    where
        Req: Send + 'static,
        Resp: Send + 'static,
        F: Fn(ServerHandlerContext, ServerRequestSingle<Req>, ServerResponseSink<Resp>) -> result::Result<()>
            + Send
            + 'static,
    {
        MethodHandlerServerStreaming { f: Arc::new(f) }
    }
}

impl<F> MethodHandlerBidi<F> {
    pub fn new<Req, Resp>(f: F) -> Self
    where
        Req: Send + 'static,
        Resp: Send + 'static,
        F: Fn(ServerHandlerContext, ServerRequest<Req>, ServerResponseSink<Resp>)
                -> result::Result<()>
            + Send
            + 'static,
    {
        MethodHandlerBidi { f: Arc::new(f) }
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerUnary<F>
where
    Req: Send + 'static,
    Resp: Send + 'static,
    F: Fn(ServerHandlerContext, ServerRequestSingle<Req>, ServerResponseUnarySink<Resp>) -> result::Result<()>
        + Send
        + Sync
        + 'static,
{
    fn handle(
        &self,
        ctx: ServerHandlerContext,
        req: ServerRequest<Req>,
        resp: ServerResponseSink<Resp>,
    ) -> result::Result<()> {
        struct HandlerImpl<F, Req, Resp>
        where
            Req: Send + 'static,
            Resp: Send + 'static,
            F: Fn(ServerHandlerContext, ServerRequestSingle<Req>, ServerResponseUnarySink<Resp>)
                    -> result::Result<()>
                + Send
                + Sync
                + 'static,
        {
            ctx: ServerHandlerContext,
            f: Arc<F>,
            resp: ServerResponseSink<Resp>,
            _marker: marker::PhantomData<Req>,
        }

        impl<F, Req, Resp> ServerRequestUnaryHandler<Req> for Option<HandlerImpl<F, Req, Resp>>
        where
            Req: Send + 'static,
            Resp: Send + 'static,
            F: Fn(ServerHandlerContext, ServerRequestSingle<Req>, ServerResponseUnarySink<Resp>)
                    -> result::Result<()>
                + Send
                + Sync
                + 'static,
        {
            fn grpc_message(&mut self, message: Req) -> result::Result<()> {
                let HandlerImpl {
                    ctx,
                    f,
                    resp,
                    _marker,
                } = self.take().unwrap();
                let metadata = ctx.metadata.clone();
                let req = ServerRequestSingle {
                    metadata,
                    message,
                };
                let resp = ServerResponseUnarySink { sink: resp };
                f(ctx, req, resp)
            }
        }

        req.register_unary_handler(Some(HandlerImpl {
            ctx,
            f: self.f.clone(),
            resp,
            _marker: marker::PhantomData,
        }));

        Ok(())
    }
}

impl<Req: Send + 'static, Resp: Send + 'static, F> MethodHandler<Req, Resp>
    for MethodHandlerClientStreaming<F>
where
    Resp: Send + 'static,
    F: Fn(ServerHandlerContext, ServerRequest<Req>, ServerResponseUnarySink<Resp>)
            -> result::Result<()>
        + Send
        + Sync
        + 'static,
{
    fn handle(
        &self,
        ctx: ServerHandlerContext,
        req: ServerRequest<Req>,
        resp: ServerResponseSink<Resp>,
    ) -> result::Result<()> {
        let resp = ServerResponseUnarySink { sink: resp };
        (self.f)(ctx, req, resp)
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerServerStreaming<F>
where
    Req: Send + 'static,
    Resp: Send + 'static,
    F: Fn(ServerHandlerContext, ServerRequestSingle<Req>, ServerResponseSink<Resp>) -> result::Result<()>
        + Send
        + Sync
        + 'static,
{
    fn handle(
        &self,
        ctx: ServerHandlerContext,
        req: ServerRequest<Req>,
        resp: ServerResponseSink<Resp>,
    ) -> result::Result<()> {
        struct HandlerImpl<F, Req, Resp>
        where
            Req: Send + 'static,
            Resp: Send + 'static,
            F: Fn(ServerHandlerContext, ServerRequestSingle<Req>, ServerResponseSink<Resp>)
                    -> result::Result<()>
                + Send
                + Sync
                + 'static,
        {
            ctx: ServerHandlerContext,
            f: Arc<F>,
            resp: ServerResponseSink<Resp>,
            _marker: marker::PhantomData<Req>,
        }

        impl<F, Req, Resp> ServerRequestUnaryHandler<Req> for Option<HandlerImpl<F, Req, Resp>>
        where
            Req: Send + 'static,
            Resp: Send + 'static,
            F: Fn(ServerHandlerContext, ServerRequestSingle<Req>, ServerResponseSink<Resp>)
                    -> result::Result<()>
                + Send
                + Sync
                + 'static,
        {
            fn grpc_message(&mut self, req: Req) -> result::Result<()> {
                let HandlerImpl {
                    ctx,
                    f,
                    resp,
                    _marker,
                } = self.take().unwrap();
                let metadata = ctx.metadata.clone();
                let req = ServerRequestSingle {
                    metadata,
                    message: req,
                };
                f(ctx, req, resp)
            }
        }

        req.register_unary_handler(Some(HandlerImpl {
            ctx,
            f: self.f.clone(),
            resp,
            _marker: marker::PhantomData,
        }));

        Ok(())
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerBidi<F>
where
    Req: Send + 'static,
    Resp: Send + 'static,
    F: Fn(ServerHandlerContext, ServerRequest<Req>, ServerResponseSink<Resp>) -> result::Result<()>
        + Send
        + Sync
        + 'static,
{
    fn handle(
        &self,
        ctx: ServerHandlerContext,
        req: ServerRequest<Req>,
        resp: ServerResponseSink<Resp>,
    ) -> result::Result<()> {
        (self.f)(ctx, req, resp)
    }
}

pub(crate) trait MethodHandlerDispatchUntyped {
    fn start_request(
        &self,
        ctx: ServerHandlerContext,
        req: ServerRequestUntyped,
        resp: ServerResponseUntypedSink,
    ) -> result::Result<()>;
}

struct MethodHandlerDispatchImpl<Req, Resp> {
    desc: Arc<MethodDescriptor<Req, Resp>>,
    method_handler: Box<MethodHandler<Req, Resp> + Sync + Send>,
}

impl<Req, Resp> MethodHandlerDispatchUntyped for MethodHandlerDispatchImpl<Req, Resp>
where
    Req: Send + 'static,
    Resp: Send + 'static,
{
    fn start_request(
        &self,
        ctx: ServerHandlerContext,
        req: ServerRequestUntyped,
        resp: ServerResponseUntypedSink,
    ) -> result::Result<()> {
        let req = ServerRequest {
            req,
            marshaller: self.desc.req_marshaller.clone(),
        };

        let resp = ServerResponseSink {
            common: SinkCommon {
                marshaller: self.desc.resp_marshaller.clone(),
                sink: resp,
            },
        };

        // TODO: catch unwind for better diag
        self.method_handler.handle(ctx, req, resp)
        //        let resp = catch_unwind(AssertUnwindSafe(|| {
        //
        //        }));
        //        match resp {
        //            Ok(resp) => {
        //                let desc_copy = self.desc.clone();
        //                resp.and_then_items(move |resp| desc_copy.resp_marshaller.write(&resp))
        //            }
        //            Err(e) => {
        //                let message = any_to_string(e);
        //                StreamingResponse::err(Error::Panic(message))
        //            }
        //        }
    }
}

pub struct ServerMethod {
    pub(crate) name: String,
    pub(crate) dispatch: Box<MethodHandlerDispatchUntyped + Sync + Send>,
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
