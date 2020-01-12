pub(crate) mod ctx;
pub(crate) mod method;
pub(crate) mod req_handler;
pub(crate) mod req_handler_unary;
pub(crate) mod req_single;
pub(crate) mod req_stream;
pub(crate) mod resp_sink;
pub(crate) mod resp_sink_untyped;
pub(crate) mod resp_unary_sink;
pub(crate) mod types;

use std::sync::Arc;

use httpbis;

use result::Result;

use tls_api;
use tls_api_stub;

use common::sink::SinkCommonUntyped;
use httpbis::AnySocketAddr;
use proto::grpc_status::GrpcStatus;
use proto::headers::grpc_error_message;
use result;
use server::ctx::ServerHandlerContext;
use server::method::ServerMethod;
use server::req_handler::ServerRequestUntyped;
use server::resp_sink_untyped::ServerResponseUntypedSink;
use Metadata;

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
        self.methods.iter().filter(|m| m.name == name).next()
    }

    pub(crate) fn handle_method(
        &self,
        name: &str,
        ctx: ServerHandlerContext,
        req: ServerRequestUntyped,
        mut resp: ServerResponseUntypedSink,
    ) -> result::Result<()> {
        match self.find_method(name) {
            Some(method) => method.dispatch.start_request(ctx, req, resp),
            None => {
                resp.send_grpc_error(GrpcStatus::Unimplemented, "Unimplemented method".to_owned())?;
                Ok(())
            }
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct ServerConf {}

impl ServerConf {
    pub fn new() -> ServerConf {
        Default::default()
    }
}

pub struct ServerBuilder<A: tls_api::TlsAcceptor = tls_api_stub::TlsAcceptor> {
    pub http: httpbis::ServerBuilder<A>,
    pub conf: ServerConf,
}

impl ServerBuilder<tls_api_stub::TlsAcceptor> {
    pub fn new_plain() -> ServerBuilder {
        ServerBuilder::new()
    }
}

impl<A: tls_api::TlsAcceptor> ServerBuilder<A> {
    pub fn new() -> ServerBuilder<A> {
        ServerBuilder {
            http: httpbis::ServerBuilder::new(),
            conf: ServerConf::new(),
        }
    }

    #[cfg(unix)]
    pub fn new_unix() -> ServerBuilder<A> {
        ServerBuilder {
            http: httpbis::ServerBuilder::new(),
            conf: ServerConf::new(),
        }
    }

    pub fn add_service(&mut self, def: ServerServiceDefinition) {
        self.http.service.set_service(
            &def.prefix.clone(),
            Arc::new(GrpcServerHandler {
                service_definition: Arc::new(def),
            }),
        );
    }

    pub fn build(mut self) -> Result<Server> {
        self.http.conf.thread_name = Some(
            self.http
                .conf
                .thread_name
                .unwrap_or_else(|| "grpc-server-loop".to_owned()),
        );

        Ok(Server {
            server: self.http.build()?,
        })
    }
}

#[derive(Debug)]
pub struct Server {
    server: httpbis::Server,
}

impl Server {
    pub fn local_addr(&self) -> &AnySocketAddr {
        self.server.local_addr()
    }

    pub fn is_alive(&self) -> bool {
        self.server.is_alive()
    }
}

/// Implementation of gRPC over http2 HttpService
struct GrpcServerHandler {
    service_definition: Arc<ServerServiceDefinition>,
}

impl httpbis::ServerHandler for GrpcServerHandler {
    fn start_request(
        &self,
        context: httpbis::ServerHandlerContext,
        req: httpbis::ServerRequest,
        mut resp: httpbis::ServerResponse,
    ) -> httpbis::Result<()> {
        // TODO: clone
        let path = req.headers.path().to_owned();

        // TODO: clone
        let metadata = match Metadata::from_headers(req.headers.clone()) {
            Ok(metadata) => metadata,
            Err(_) => {
                resp.send_message(grpc_error_message("decode metadata error"))?;
                return Ok(());
            }
        };

        let req = ServerRequestUntyped { req };

        resp.set_drop_callback(move |resp| {
            Ok(resp.send_message(grpc_error_message(
                "grpc server handler did not close the sender",
            ))?)
        });

        let resp = ServerResponseUntypedSink {
            common: SinkCommonUntyped { http: resp },
        };

        let context = ServerHandlerContext {
            ctx: context,
            metadata,
        };

        // TODO: catch unwind
        self.service_definition
            .handle_method(&path, context, req, resp)?;

        Ok(())
    }
}
