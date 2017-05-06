// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]


// interface

pub trait Greeter {
    fn SayHello(&self, m: ::grpc::GrpcMetadata, p: super::helloworld::HelloRequest) -> ::grpc::result::GrpcResult<super::helloworld::HelloReply>;
}

pub trait GreeterAsync {
    fn SayHello(&self, m: ::grpc::GrpcMetadata, p: super::helloworld::HelloRequest) -> ::grpc::GrpcSingleResponse<super::helloworld::HelloReply>;
}

// sync client

pub struct GreeterClient {
    async_client: GreeterAsyncClient,
}

impl GreeterClient {
    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        GreeterAsyncClient::new(host, port, tls, conf).map(|c| {
            GreeterClient {
                async_client: c,
            }
        })
    }
}

impl Greeter for GreeterClient {
    fn SayHello(&self, m: ::grpc::GrpcMetadata, p: super::helloworld::HelloRequest) -> ::grpc::result::GrpcResult<super::helloworld::HelloReply> {
        ::grpc::GrpcSingleResponse::wait_drop_metadata(self.async_client.SayHello(m, p))
    }
}

// async client

pub struct GreeterAsyncClient {
    grpc_client: ::grpc::client::GrpcClient,
    method_SayHello: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::helloworld::HelloRequest, super::helloworld::HelloReply>>,
}

impl GreeterAsyncClient {
    pub fn with_client(grpc_client: ::grpc::client::GrpcClient) -> Self {
        GreeterAsyncClient {
            grpc_client: grpc_client,
            method_SayHello: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                name: "/helloworld.Greeter/SayHello".to_string(),
                streaming: ::grpc::method::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
            }),
        }
    }

    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        ::grpc::client::GrpcClient::new(host, port, tls, conf).map(|c| {
            GreeterAsyncClient::with_client(c)
        })
    }
}

impl GreeterAsync for GreeterAsyncClient {
    fn SayHello(&self, m: ::grpc::GrpcMetadata, p: super::helloworld::HelloRequest) -> ::grpc::GrpcSingleResponse<super::helloworld::HelloReply> {
        self.grpc_client.call_unary(m, p, self.method_SayHello.clone())
    }
}

// sync server

pub struct GreeterServer {
    async_server: GreeterAsyncServer,
}

impl ::std::ops::Deref for GreeterServer {
    type Target = GreeterAsyncServer;

    fn deref(&self) -> &Self::Target {
        &self.async_server
    }
}

struct GreeterServerHandlerToAsync {
    handler: ::std::sync::Arc<Greeter + Send + Sync>,
    cpupool: ::futures_cpupool::CpuPool,
}

impl GreeterAsync for GreeterServerHandlerToAsync {
    fn SayHello(&self, m: ::grpc::GrpcMetadata, p: super::helloworld::HelloRequest) -> ::grpc::GrpcSingleResponse<super::helloworld::HelloReply> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_unary(&self.cpupool, p, move |p| {
            h.SayHello(m, p)
        })
    }
}

impl GreeterServer {
    pub fn new_plain<A : ::std::net::ToSocketAddrs, H : Greeter + Send + Sync + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let h = GreeterServerHandlerToAsync {
            cpupool: ::futures_cpupool::CpuPool::new_num_cpus(),
            handler: ::std::sync::Arc::new(h),
        };
        GreeterServer {
            async_server: GreeterAsyncServer::new(addr, conf, h),
        }
    }
}

// async server

pub struct GreeterAsyncServer {
    pub grpc_server: ::grpc::server::GrpcServer,
}

impl ::std::ops::Deref for GreeterAsyncServer {
    type Target = ::grpc::server::GrpcServer;

    fn deref(&self) -> &Self::Target {
        &self.grpc_server
    }
}

impl GreeterAsyncServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : GreeterAsync + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let service_definition = GreeterAsyncServer::new_service_def(h);
        GreeterAsyncServer {
            grpc_server: ::grpc::server::GrpcServer::new_plain(addr, conf, service_definition),
        }
    }

    pub fn new_service_def<H : GreeterAsync + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::server::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::server::ServerServiceDefinition::new(
            vec![
                ::grpc::server::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                        name: "/helloworld.Greeter/SayHello".to_string(),
                        streaming: ::grpc::method::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerUnary::new(move |m, p| handler_copy.SayHello(m, p))
                    },
                ),
            ],
        )
    }
}
