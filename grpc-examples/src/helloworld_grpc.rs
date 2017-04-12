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
    fn say_hello(&self, o: ::grpc::GrpcRequestOptions, p: super::helloworld::HelloRequest) -> ::grpc::GrpcSingleResponse<super::helloworld::HelloReply>;
}

// client

pub struct GreeterClient {
    grpc_client: ::grpc::client::GrpcClient,
    method_SayHello: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::helloworld::HelloRequest, super::helloworld::HelloReply>>,
}

impl GreeterClient {
    pub fn with_client(grpc_client: ::grpc::client::GrpcClient) -> Self {
        GreeterClient {
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
            GreeterClient::with_client(c)
        })
    }
}

impl Greeter for GreeterClient {
    fn say_hello(&self, o: ::grpc::GrpcRequestOptions, p: super::helloworld::HelloRequest) -> ::grpc::GrpcSingleResponse<super::helloworld::HelloReply> {
        self.grpc_client.call_unary(o, p, self.method_SayHello.clone())
    }
}

// server

pub struct GreeterServer {
    pub grpc_server: ::grpc::server::GrpcServer,
}

impl ::std::ops::Deref for GreeterServer {
    type Target = ::grpc::server::GrpcServer;

    fn deref(&self) -> &Self::Target {
        &self.grpc_server
    }
}

impl GreeterServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : Greeter + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let service_definition = GreeterServer::new_service_def(h);
        GreeterServer {
            grpc_server: ::grpc::server::GrpcServer::new_plain(addr, conf, service_definition),
        }
    }

    pub fn new_pool<A : ::std::net::ToSocketAddrs, H : Greeter + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H, cpu_pool: ::futures_cpupool::CpuPool) -> Self {
        let service_definition = GreeterServer::new_service_def(h);
        GreeterServer {
            grpc_server: ::grpc::server::GrpcServer::new_plain_pool(addr, conf, service_definition, cpu_pool),
        }
    }

    pub fn new_service_def<H : Greeter + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::server::ServerServiceDefinition {
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
                        ::grpc::server::MethodHandlerUnary::new(move |o, p| handler_copy.say_hello(o, p))
                    },
                ),
            ],
        )
    }
}
