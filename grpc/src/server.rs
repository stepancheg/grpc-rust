use std::sync::Arc;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;

use futures_cpupool::CpuPool;

use bytes::Bytes;

use httpbis;
use httpbis::Header;
use httpbis::Headers;
use httpbis::HttpPartStream;
use httpbis::stream_part::HttpStreamPart;

use stream_item::ItemOrMetadata;

use result::Result;

use tls_api;
use tls_api_stub;

use futures::Future;
use futures::stream;
use futures::stream::Stream;

use error::*;
use grpc::*;
use grpc_frame::*;
use req::*;
use resp::*;
use metadata::Metadata;
use server_method::*;


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
