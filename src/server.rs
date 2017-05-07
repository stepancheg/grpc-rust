use std::sync::Arc;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;

use bytes::Bytes;

use httpbis::HttpError;
use httpbis::Header;
use httpbis::Headers;
use httpbis::server::HttpServer;
use httpbis::server::ServerTlsOption;

use futures;
use futures::Future;
use futures::stream;
use futures::stream::Stream;

use method::*;
use error::*;
use futures_grpc::*;
use httpbis::futures_misc::*;
use grpc::*;
use grpc_frame::*;
use httpbis::http_common::*;
use httpbis::server_conf::*;
use httpbis::misc::any_to_string;
use req::GrpcRequestOptions;
use resp::*;
use metadata::GrpcMetadata;


pub trait MethodHandler<Req, Resp>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
{
    fn handle(&self, m: GrpcRequestOptions, req: GrpcStreamSend<Req>) -> GrpcStreamingResponse<Resp>;
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
        where
            Req : Send + 'static,
            Resp : Send + 'static,
            F : Fn(GrpcRequestOptions, Req) -> GrpcSingleResponse<Resp> + Send + 'static,
    {
        MethodHandlerUnary {
            f: Arc::new(f),
        }
    }
}

impl<F> MethodHandlerClientStreaming<F> {
    pub fn new<Req, Resp>(f: F) -> Self
        where
            Req : Send + 'static,
            Resp : Send + 'static,
            F : Fn(GrpcRequestOptions, GrpcStreamSend<Req>) -> GrpcSingleResponse<Resp> + Send + 'static,
    {
        MethodHandlerClientStreaming {
            f: Arc::new(f),
        }
    }
}

impl<F> MethodHandlerServerStreaming<F> {
    pub fn new<Req, Resp>(f: F) -> Self
        where
            Req : Send + 'static,
            Resp : Send + 'static,
            F : Fn(GrpcRequestOptions, Req) -> GrpcStreamingResponse<Resp> + Send + 'static,
    {
        MethodHandlerServerStreaming {
            f: Arc::new(f),
        }
    }
}

impl<F> MethodHandlerBidi<F> {
    pub fn new<Req, Resp>(f: F) -> Self
        where
            Req : Send + 'static,
            Resp : Send + 'static,
            F : Fn(GrpcRequestOptions, GrpcStreamSend<Req>) -> GrpcStreamingResponse<Resp> + Send + 'static,
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
        F : Fn(GrpcRequestOptions, Req) -> GrpcSingleResponse<Resp> + Send + Sync + 'static,
{
    fn handle(&self, m: GrpcRequestOptions, req: GrpcStreamSend<Req>) -> GrpcStreamingResponse<Resp> {
        let f = self.f.clone();
        GrpcSingleResponse(
            Box::new(stream_single(req)
                .and_then(move |req| f(m, req).0)))
                    .into_stream()
    }
}

impl<Req : Send + 'static, Resp : Send + 'static, F> MethodHandler<Req, Resp> for MethodHandlerClientStreaming<F>
    where
        Resp : Send + 'static,
        F : Fn(GrpcRequestOptions, GrpcStreamSend<Req>) -> GrpcSingleResponse<Resp> + Send + Sync + 'static,
{
    fn handle(&self, m: GrpcRequestOptions, req: GrpcStreamSend<Req>) -> GrpcStreamingResponse<Resp> {
        ((self.f)(m, req)).into_stream()
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerServerStreaming<F>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
        F : Fn(GrpcRequestOptions, Req) -> GrpcStreamingResponse<Resp> + Send + Sync + 'static,
{
    fn handle(&self, o: GrpcRequestOptions, req: GrpcStreamSend<Req>) -> GrpcStreamingResponse<Resp> {
        let f = self.f.clone();
        GrpcStreamingResponse(Box::new(
            stream_single(req)
                .and_then(move |req| f(o, req).0)))
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerBidi<F>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
        F : Fn(GrpcRequestOptions, GrpcStreamSend<Req>) -> GrpcStreamingResponse<Resp> + Send + Sync + 'static,
{
    fn handle(&self, m: GrpcRequestOptions, req: GrpcStreamSend<Req>) -> GrpcStreamingResponse<Resp> {
        (self.f)(m, req)
    }
}


trait MethodHandlerDispatch {
    fn start_request(&self, m: GrpcRequestOptions, grpc_frames: GrpcStreamSend<Vec<u8>>) -> GrpcStreamSend<Vec<u8>>;
}

struct MethodHandlerDispatchImpl<Req, Resp> {
    desc: Arc<MethodDescriptor<Req, Resp>>,
    method_handler: Box<MethodHandler<Req, Resp> + Sync + Send>,
}

impl<Req, Resp> MethodHandlerDispatch for MethodHandlerDispatchImpl<Req, Resp>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
{
    fn start_request(&self, o: GrpcRequestOptions, req_grpc_frames: GrpcStreamSend<Vec<u8>>) -> GrpcStreamSend<Vec<u8>> {
        let desc = self.desc.clone();
        let req = req_grpc_frames.and_then(move |frame| desc.req_marshaller.read(&frame));
        let resp =
            catch_unwind(AssertUnwindSafe(|| self.method_handler.handle(o, Box::new(req))));
        match resp {
            Ok(resp) => {
                let desc_copy = self.desc.clone();
                // TODO: keep metadata
                Box::new(resp.drop_metadata()
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

    /// Join multiple service definitions into one
    pub fn join<I>(iter: I) -> ServerServiceDefinition
        where I : IntoIterator<Item=ServerServiceDefinition>
    {
        ServerServiceDefinition {
            methods: iter.into_iter().flat_map(|s| s.methods).collect()
        }
    }

    pub fn find_method(&self, name: &str) -> &ServerMethod {
        self.methods.iter()
            .filter(|m| m.name == name)
            .next()
            .expect(&format!("unknown method: {}", name))
    }

    pub fn handle_method(&self, name: &str, o: GrpcRequestOptions, message: GrpcStreamSend<Vec<u8>>) -> GrpcStreamSend<Vec<u8>> {
        self.find_method(name).dispatch.start_request(o, message)
    }
}

#[derive(Default, Debug, Clone)]
pub struct GrpcServerConf {
    pub http: HttpServerConf,
}


pub struct GrpcServer {
    server: HttpServer,
}

impl GrpcServer {
    pub fn new_plain<A : ToSocketAddrs>(
        addr: A,
        conf: GrpcServerConf,
        service_definition: ServerServiceDefinition)
            -> GrpcServer
    {
        GrpcServer::new(addr, ServerTlsOption::Plain, conf, service_definition)
    }

    pub fn new<A : ToSocketAddrs>(
        addr: A,
        tls: ServerTlsOption,
        conf: GrpcServerConf,
        service_definition: ServerServiceDefinition)
            -> GrpcServer
    {
        let mut conf = conf;
        conf.http.thread_name =
            Some(conf.http.thread_name.unwrap_or_else(|| "grpc-server-loop".to_owned()));

        let service_definition = Arc::new(service_definition);
        GrpcServer {
            server: HttpServer::new(addr, tls, conf.http, GrpcHttpServerHandlerFactory {
                service_definition: service_definition.clone(),
            })
        }
    }

    pub fn local_addr(&self) -> &SocketAddr {
        self.server.local_addr()
    }

    pub fn is_alive(&self) -> bool {
        self.server.is_alive()
    }
}

struct GrpcHttpServerHandlerFactory {
    service_definition: Arc<ServerServiceDefinition>,
}


fn stream_500(message: &str) -> HttpResponse {
    // TODO: HttpResponse::headers
    HttpResponse::from_stream(stream::once(Ok(HttpStreamPart::last_headers(Headers(vec![
        Header::new(":status", "500"),
        Header::new(HEADER_GRPC_MESSAGE, message.to_owned()),
    ])))))
}

impl HttpService for GrpcHttpServerHandlerFactory {
    fn new_request(&self, headers: Headers, req: HttpPartFutureStreamSend) -> HttpResponse {

        let path = match headers.get_opt(":path") {
            Some(path) => path.to_owned(),
            None => return stream_500("no :path header"),
        };

        let grpc_request = GrpcFrameFromHttpFramesStreamRequest::new(req);

        let metadata = match GrpcMetadata::from_headers(headers) {
            Ok(metadata) => metadata,
            Err(_) => return stream_500("decode metadata error"),
        };

        // TODO: catch unwind
        let grpc_frames = self.service_definition.handle_method(
            &path,
            GrpcRequestOptions { metadata: metadata },
            Box::new(grpc_request));

        let s1 = stream::once(Ok(HttpStreamPart::intermediate_headers(Headers(vec![
            Header::new(":status", "200"),
            Header::new("content-type", "application/grpc"),
        ]))));

        let s2 = grpc_frames
            .map(|frame| HttpStreamPart::intermediate_data(Bytes::from(write_grpc_frame_to_vec(&frame))))
            .then(|result| {
                match result {
                    Ok(part) => {
                        let r: Result<_, HttpError> = Ok(part);
                        r
                    }
                    Err(e) =>
                        Ok(HttpStreamPart::last_headers(
                            match e {
                                GrpcError::GrpcMessage(GrpcMessageError { grpc_status, grpc_message }) => {
                                    Headers(vec![
                                        Header::new(":status", "500"),
                                        // TODO: check nonzero
                                        Header::new(HEADER_GRPC_STATUS, format!("{}", grpc_status)),
                                        // TODO: escape invalid
                                        Header::new(HEADER_GRPC_MESSAGE, grpc_message),
                                    ])
                                }
                                e => {
                                    Headers(vec![
                                        Header::new(":status", "500"),
                                        Header::new(HEADER_GRPC_MESSAGE, format!("error: {:?}", e)),
                                    ])
                                }
                            }
                        ))
                }
            });

        let s3 = stream::once(Ok(HttpStreamPart::last_headers(Headers(vec![
            Header::new(HEADER_GRPC_STATUS, "0"),
        ]))));

        let http_parts = s1.chain(s2).chain(s3);

        HttpResponse::from_stream(http_parts)
    }
}
