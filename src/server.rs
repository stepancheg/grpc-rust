use std::sync::Arc;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;

use futures_cpupool::CpuPool;

use bytes::Bytes;

use httpbis;
use httpbis::Header;
use httpbis::Headers;
use httpbis::misc::any_to_string;
use httpbis::futures_misc::*;
use httpbis::HttpPartStream;
use httpbis::stream_part::HttpStreamPart;

use stream_item::ItemOrMetadata;

use result::Result;

use tls_api;
use tls_api_stub;

use futures::Future;
use futures::stream;
use futures::stream::Stream;

use method::*;
use error::*;
use grpc::*;
use grpc_frame::*;
use req::*;
use resp::*;
use metadata::Metadata;


pub trait MethodHandler<Req, Resp>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
{
    fn handle(&self, m: RequestOptions, req: StreamingRequest<Req>) -> StreamingResponse<Resp>;
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
            F : Fn(RequestOptions, Req) -> SingleResponse<Resp> + Send + 'static,
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
            F : Fn(RequestOptions, StreamingRequest<Req>) -> SingleResponse<Resp> + Send + 'static,
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
            F : Fn(RequestOptions, Req) -> StreamingResponse<Resp> + Send + 'static,
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
            F : Fn(RequestOptions, StreamingRequest<Req>) -> StreamingResponse<Resp> + Send + 'static,
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
        F : Fn(RequestOptions, Req) -> SingleResponse<Resp> + Send + Sync + 'static,
{
    fn handle(&self, m: RequestOptions, req: StreamingRequest<Req>) -> StreamingResponse<Resp> {
        let f = self.f.clone();
        SingleResponse::new(
            stream_single(req.0).and_then(move |req| f(m, req).0))
                .into_stream()
    }
}

impl<Req : Send + 'static, Resp : Send + 'static, F> MethodHandler<Req, Resp> for MethodHandlerClientStreaming<F>
    where
        Resp : Send + 'static,
        F : Fn(RequestOptions, StreamingRequest<Req>) -> SingleResponse<Resp> + Send + Sync + 'static,
{
    fn handle(&self, m: RequestOptions, req: StreamingRequest<Req>) -> StreamingResponse<Resp> {
        ((self.f)(m, req)).into_stream()
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerServerStreaming<F>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
        F : Fn(RequestOptions, Req) -> StreamingResponse<Resp> + Send + Sync + 'static,
{
    fn handle(&self, o: RequestOptions, req: StreamingRequest<Req>) -> StreamingResponse<Resp> {
        let f = self.f.clone();
        StreamingResponse(Box::new(
            stream_single(req.0)
                .and_then(move |req| f(o, req).0)))
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerBidi<F>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
        F : Fn(RequestOptions, StreamingRequest<Req>) -> StreamingResponse<Resp> + Send + Sync + 'static,
{
    fn handle(&self, m: RequestOptions, req: StreamingRequest<Req>) -> StreamingResponse<Resp> {
        (self.f)(m, req)
    }
}


trait MethodHandlerDispatch {
    fn start_request(&self, m: RequestOptions, grpc_frames: StreamingRequest<Bytes>)
                     -> StreamingResponse<Vec<u8>>;
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
    fn start_request(&self, o: RequestOptions, req_grpc_frames: StreamingRequest<Bytes>)
                     -> StreamingResponse<Vec<u8>>
    {
        let desc = self.desc.clone();
        let req = req_grpc_frames.0.and_then(move |frame| desc.req_marshaller.read(frame));
        let resp =
            catch_unwind(AssertUnwindSafe(|| self.method_handler.handle(o, StreamingRequest::new(req))));
        match resp {
            Ok(resp) => {
                let desc_copy = self.desc.clone();
                resp.and_then_items(move |resp| {
                    desc_copy.resp_marshaller.write(&resp)
                })
            }
            Err(e) => {
                let message = any_to_string(e);
                StreamingResponse::err(Error::Panic(message))
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
    pub prefix: String,
    pub methods: Vec<ServerMethod>,
}

impl ServerServiceDefinition {
    pub fn new(prefix: &str, methods: Vec<ServerMethod>) -> ServerServiceDefinition {
        ServerServiceDefinition {
            prefix: prefix.to_owned(),
            methods: methods,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<&ServerMethod> {
        self.methods.iter()
            .filter(|m| m.name == name)
            .next()
    }

    pub fn handle_method(&self, name: &str, o: RequestOptions, message: StreamingRequest<Bytes>)
        -> StreamingResponse<Vec<u8>>
    {
        match self.find_method(name) {
            Some(method) => method.dispatch.start_request(o, message),
            None => {
                StreamingResponse::no_metadata(Box::new(stream::once(Err(
                    Error::GrpcMessage(
                        GrpcMessageError {
                            grpc_status: GrpcStatus::Unimplemented as i32,
                            grpc_message: String::from("Unimplemented method"),
                        }
                    )
                ))))
            },
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct ServerConf {
}

impl ServerConf {
    pub fn new() -> ServerConf {
        Default::default()
    }
}

pub struct ServerBuilder<A : tls_api::TlsAcceptor = tls_api_stub::TlsAcceptor> {
    pub http: httpbis::ServerBuilder<A>,
    pub conf: ServerConf,
}

impl ServerBuilder<tls_api_stub::TlsAcceptor> {
    pub fn new_plain() -> ServerBuilder {
        ServerBuilder::new()
    }
}

impl<A : tls_api::TlsAcceptor> ServerBuilder<A> {
    pub fn new() -> ServerBuilder<A> {
        ServerBuilder {
            http: httpbis::ServerBuilder::new(),
            conf: ServerConf::new(),
        }
    }

    pub fn add_service(&mut self, def: ServerServiceDefinition) {
        self.http.service.set_service(&def.prefix.clone(), Arc::new(GrpcHttpService {
            service_definition: Arc::new(def),
        }));
    }

    pub fn build(mut self) -> Result<Server> {
        self.http.conf.thread_name =
            Some(self.http.conf.thread_name.unwrap_or_else(|| "grpc-server-loop".to_owned()));

        Ok(Server {
            server: self.http.build()?,
        })
    }
}


pub struct Server {
    server: httpbis::Server,
}

impl Server {
    /// Without TLS
    pub fn new_plain<A : ToSocketAddrs>(
        addr: A,
        conf: ServerConf,
        def: ServerServiceDefinition)
            -> Result<Server>
    {
        let mut server = ServerBuilder::new_plain();
        server.conf = conf;
        server.http.set_addr(addr)?;
        server.add_service(def);
        server.build()
    }

    /// Without TLS and execute handler in given CpuPool
    pub fn new_plain_pool<A : ToSocketAddrs>(
        addr: A,
        conf: ServerConf,
        def: ServerServiceDefinition,
        cpu_pool: CpuPool)
            -> Result<Server>
    {
        let mut server = ServerBuilder::new_plain();
        server.conf = conf;
        server.http.set_addr(addr)?;
        server.add_service(def);
        server.http.cpu_pool = httpbis::CpuPoolOption::CpuPool(cpu_pool);
        server.build()
    }

    pub fn new_single_thread<A : ToSocketAddrs, S : tls_api::TlsAcceptor>(
        addr: A,
        tls: httpbis::ServerTlsOption<S>,
        conf: ServerConf,
        def: ServerServiceDefinition)
            -> Result<Server>
    {
        let mut server = ServerBuilder::new();
        server.conf = conf;
        server.http.set_addr(addr)?;
        server.http.tls = tls;
        server.add_service(def);
        server.build()
    }

    pub fn new_pool<A : ToSocketAddrs, S : tls_api::TlsAcceptor>(
        addr: A,
        tls: httpbis::ServerTlsOption<S>,
        conf: ServerConf,
        def: ServerServiceDefinition,
        cpu_pool: CpuPool)
            -> Result<Server>
    {
        let mut server = ServerBuilder::new();
        server.conf = conf;
        server.http.set_addr(addr)?;
        server.http.cpu_pool = httpbis::CpuPoolOption::CpuPool(cpu_pool);
        server.http.tls = tls;
        server.add_service(def);
        server.build()
    }

    pub fn local_addr(&self) -> &SocketAddr {
        self.server.local_addr()
    }

    pub fn is_alive(&self) -> bool {
        self.server.is_alive()
    }
}

/// Implementation of gRPC over http2 HttpService
struct GrpcHttpService {
    service_definition: Arc<ServerServiceDefinition>,
}


/// Create HTTP response for gRPC error
fn http_response_500(message: &str) -> httpbis::Response {
    // TODO: HttpResponse::headers
    let headers = Headers(vec![
        Header::new(":status", "500"),
        Header::new(HEADER_GRPC_MESSAGE, message.to_owned()),
    ]);
    httpbis::Response::headers_and_stream(headers, httpbis::HttpPartStream::empty())
}

impl httpbis::Service for GrpcHttpService {
    fn start_request(&self, headers: Headers, req: HttpPartStream) -> httpbis::Response {

        let path = match headers.get_opt(":path") {
            Some(path) => path.to_owned(),
            None => return http_response_500("no :path header"),
        };

        let grpc_request = GrpcFrameFromHttpFramesStreamRequest::new(req);

        let metadata = match Metadata::from_headers(headers) {
            Ok(metadata) => metadata,
            Err(_) => return http_response_500("decode metadata error"),
        };

        let request_options = RequestOptions { metadata: metadata };
        // TODO: catch unwind
        let grpc_response = self.service_definition.handle_method(
            &path, request_options, StreamingRequest::new(grpc_request));

        httpbis::Response::new(grpc_response.0.map_err(httpbis::Error::from).map(|(metadata, grpc_frames)| {
            let mut init_headers = Headers(vec![
                Header::new(":status", "200"),
                Header::new("content-type", "application/grpc"),
            ]);

            init_headers.extend(metadata.into_headers());

            let s2 = grpc_frames
                .map_items(|frame| HttpStreamPart::intermediate_data(Bytes::from(write_grpc_frame_to_vec(&frame))))
                .then_items(|result| {
                    match result {
                        Ok(part) => {
                            Ok(part)
                        }
                        Err(e) =>
                            Ok(HttpStreamPart::last_headers(
                                match e {
                                    Error::GrpcMessage(GrpcMessageError { grpc_status, grpc_message }) => {
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
                })
                .0.map(|item| {
                    match item {
                        ItemOrMetadata::Item(part) => part,
                        ItemOrMetadata::TrailingMetadata(trailing_metadata) => {
                            HttpStreamPart::last_headers( {
                                let mut trailing = Headers(vec![
                                    Header::new(HEADER_GRPC_STATUS, "0")
                                ]);
                                trailing.extend(trailing_metadata.into_headers());
                                trailing
                            })
                        },
                    }
                })
                .map_err(httpbis::Error::from);

            // If stream contains trailing metadata, it is converted to grpc status 0,
            // here we add trailing headers after trailing headers are ignored by HTTP server.
            let s3 = stream::once(Ok(HttpStreamPart::last_headers(Headers(vec![
                Header::new(HEADER_GRPC_STATUS, "0"),
            ]))));

            let http_parts = HttpPartStream::new(s2.chain(s3));

            (init_headers, http_parts)
        }))
    }
}
