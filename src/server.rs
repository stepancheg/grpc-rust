use std::sync::Arc;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;
use std::any::Any;

use http2::HttpError;
use http2::Header;
use http2::StaticHeader;
use http2::server::Http2Server;

use futures;
use futures::Future;
use futures::stream;
use futures::stream::Stream;

use method::*;
use error::*;
use futures_grpc::*;
use http2::futures_misc::*;
use http2::solicit_misc::*;
use grpc::*;
use grpc_frame::*;
use http2::http_common::*;


pub trait MethodHandler<Req, Resp> {
    fn handle(&self, req: GrpcStreamSend<Req>) -> GrpcStreamSend<Resp>;
}

pub struct MethodHandlerUnary<F> {
    f: Arc<F>
}

pub struct MethodHandlerServerStreaming<F> {
    f: Arc<F>
}

pub struct MethodHandlerClientStreaming<F> {
    f: Arc<F>
}

pub struct MethodHandlerBidi<F> {
    f: Arc<F>
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
        where F : Fn(Req) -> GrpcFutureSend<Resp> + Send + Sync,
    {
        MethodHandlerUnary {
            f: Arc::new(f),
        }
    }
}

impl<F> MethodHandlerClientStreaming<F> {
    pub fn new<Req, Resp>(f: F) -> Self
        where F : Fn(GrpcStreamSend<Req>) -> GrpcFutureSend<Resp> + Send + Sync,
    {
        MethodHandlerClientStreaming {
            f: Arc::new(f),
        }
    }
}

impl<F> MethodHandlerServerStreaming<F> {
    pub fn new<Req, Resp>(f: F) -> Self
        where F : Fn(Req) -> GrpcStreamSend<Resp> + Send + Sync,
    {
        MethodHandlerServerStreaming {
            f: Arc::new(f),
        }
    }
}

impl<F> MethodHandlerBidi<F> {
    pub fn new<Req, Resp>(f: F) -> Self
        where F : Fn(GrpcStreamSend<Req>) -> GrpcStreamSend<Resp> + Send + Sync,
    {
        MethodHandlerBidi {
            f: Arc::new(f),
        }
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerUnary<F>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
        F : Fn(Req) -> GrpcFutureSend<Resp> + Send + Sync + 'static,
{
    fn handle(&self, req: GrpcStreamSend<Req>) -> GrpcStreamSend<Resp> {
        let f = self.f.clone();
        Box::new(
            stream_single(req)
                .and_then(move |req| f(req))
                .into_stream())
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerClientStreaming<F>
    where
        Resp : Send + 'static,
        F : Fn(GrpcStreamSend<Req>) -> GrpcFutureSend<Resp> + Send + Sync + 'static,
{
    fn handle(&self, req: GrpcStreamSend<Req>) -> GrpcStreamSend<Resp> {
        Box::new(((self.f)(req)).into_stream())
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerServerStreaming<F>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
        F : Fn(Req) -> GrpcStreamSend<Resp> + Send + Sync + 'static,
{
    fn handle(&self, req: GrpcStreamSend<Req>) -> GrpcStreamSend<Resp> {
        let f = self.f.clone();
        Box::new(
            stream_single(req)
                .map(move |req| f(req))
                .flatten_stream())
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerBidi<F>
    where
        Resp : Send + 'static,
        F : Fn(GrpcStreamSend<Req>) -> GrpcStreamSend<Resp> + Send + Sync + 'static,
{
    fn handle(&self, req: GrpcStreamSend<Req>) -> GrpcStreamSend<Resp> {
        Box::new((self.f)(req))
    }
}


trait MethodHandlerDispatch {
    fn start_request(&self, req: GrpcStreamSend<Vec<u8>>) -> GrpcStreamSend<Vec<u8>>;
}

struct MethodHandlerDispatchImpl<Req, Resp> {
    desc: Arc<MethodDescriptor<Req, Resp>>,
    method_handler: Box<MethodHandler<Req, Resp> + Sync + Send>,
}

fn any_to_string(any: Box<Any + Send + 'static>) -> String {
    if any.is::<String>() {
        *any.downcast::<String>().unwrap()
    } else if any.is::<&str>() {
        (*any.downcast::<&str>().unwrap()).to_owned()
    } else {
        "unknown any".to_owned()
    }
}

impl<Req, Resp> MethodHandlerDispatch for MethodHandlerDispatchImpl<Req, Resp>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
{
    fn start_request(&self, req_grpc_frames: GrpcStreamSend<Vec<u8>>) -> GrpcStreamSend<Vec<u8>> {
        let desc = self.desc.clone();
        let req = req_grpc_frames.and_then(move |frame| desc.req_marshaller.read(&frame));
        let resp =
            catch_unwind(AssertUnwindSafe(|| self.method_handler.handle(Box::new(req))));
        match resp {
            Ok(resp) => {
                let desc_copy = self.desc.clone();
                Box::new(resp
                    .and_then(move |resp| desc_copy.resp_marshaller.write(&resp)))
            }
            Err(e) => {
                let message = any_to_string(e);
                Box::new(futures::failed(GrpcError::Panic(message)).into_stream())
            }
        }
    }
}

pub struct ServerMethod {
    name: String,
    dispatch: Box<MethodHandlerDispatch + Sync + Send>,
}

impl ServerMethod {
    pub fn new<Req, Resp, H>(method: Arc<MethodDescriptor<Req, Resp>>, handler: H) -> ServerMethod
        where
            Req : Send + 'static,
            Resp : Send + 'static,
            H : MethodHandler<Req, Resp> + 'static + Sync + Send,
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

pub struct ServerServiceDefinition {
    methods: Vec<ServerMethod>,
}

impl ServerServiceDefinition {
    pub fn new(methods: Vec<ServerMethod>) -> ServerServiceDefinition {
        ServerServiceDefinition {
            methods: methods,
        }
    }

    pub fn find_method(&self, name: &str) -> &ServerMethod {
        self.methods.iter()
            .filter(|m| m.name == name)
            .next()
            .expect(&format!("unknown method: {}", name))
    }

    pub fn handle_method(&self, name: &str, message: GrpcStreamSend<Vec<u8>>) -> GrpcStreamSend<Vec<u8>> {
        self.find_method(name).dispatch.start_request(message)
    }
}


pub struct GrpcServer {
    server: Http2Server,
}

impl GrpcServer {
    pub fn new<A: ToSocketAddrs>(addr: A, service_definition: ServerServiceDefinition) -> GrpcServer {
        let service_definition = Arc::new(service_definition);
        GrpcServer {
            server: Http2Server::new(addr, GrpcHttpServerHandlerFactory {
                service_definition: service_definition.clone(),
            })
        }
    }

    pub fn local_addr(&self) -> &SocketAddr {
        self.server.local_addr()
    }
}

struct GrpcHttpServerHandlerFactory {
    service_definition: Arc<ServerServiceDefinition>,
}


fn stream_500(message: &str) -> HttpStreamStreamSend {
    Box::new(stream::once(Ok(HttpStreamPart::last_headers(vec![
        Header::new(":status", "500"),
        Header::new(HEADER_GRPC_MESSAGE, message.to_owned()),
    ]))))
}

impl HttpService for GrpcHttpServerHandlerFactory {
    fn new_request(&self, headers: Vec<StaticHeader>, req: HttpStreamStreamSend) -> HttpStreamStreamSend {

        let path = match slice_get_header(&headers, ":path") {
            Some(path) => path.to_owned(),
            None => return stream_500("no :path header"),
        };

        let grpc_request = GrpcFrameFromHttpFramesStreamRequest::new(req);

        // TODO: catch unwind
        let grpc_frames = self.service_definition.handle_method(&path, Box::new(grpc_request));

        let http_parts = stream_concat3(
            stream::once(Ok(HttpStreamPart::intermediate_headers(vec![
                Header::new(":status", "200"),
            ]))),
            grpc_frames
                .map(|frame| HttpStreamPart::intermediate_data(write_grpc_frame_to_vec(&frame)))
                .then(|result| {
                    match result {
                        Ok(part) => { let r: Result<_, HttpError> = Ok(part); r }
                        Err(e) =>
                            Ok(HttpStreamPart::last_headers(vec![
                                Header::new(":status", "500"),
                                Header::new(HEADER_GRPC_MESSAGE, format!("error: {:?}", e)),
                            ]))
                    }
                }),
            stream::once(Ok(HttpStreamPart::last_headers(vec![
                Header::new(HEADER_GRPC_STATUS, "0"),
            ]))));


        Box::new(http_parts)
    }
}
