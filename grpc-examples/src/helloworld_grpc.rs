// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]


// interface

pub trait Greeter {
    fn SayHello(&self, p: super::helloworld::HelloRequest) -> ::grpc::result::GrpcResult<super::helloworld::HelloReply>;
}

pub trait GreeterAsync {
    fn SayHello(&self, p: super::helloworld::HelloRequest) -> ::grpc::futures_grpc::GrpcFuture<super::helloworld::HelloReply>;
}

// sync client

pub struct GreeterClient {
    async_client: GreeterAsyncClient,
}

impl GreeterClient {
    pub fn new(host: &str, port: u16) -> Self {
        GreeterClient {
            async_client: GreeterAsyncClient::new(host, port),
        }
    }
}

impl Greeter for GreeterClient {
    fn SayHello(&self, p: super::helloworld::HelloRequest) -> ::grpc::result::GrpcResult<super::helloworld::HelloReply> {
        ::futures::Future::wait(self.async_client.SayHello(p))
    }
}

// async client

pub struct GreeterAsyncClient {
    grpc_client: ::grpc::client::GrpcClient,
}

impl GreeterAsyncClient {
    pub fn new(host: &str, port: u16) -> Self {
        GreeterAsyncClient {
            grpc_client: ::grpc::client::GrpcClient::new(host, port),
        }
    }
}

impl GreeterAsync for GreeterAsyncClient {
    fn SayHello(&self, p: super::helloworld::HelloRequest) -> ::grpc::futures_grpc::GrpcFuture<super::helloworld::HelloReply> {
        let method: ::grpc::method::MethodDescriptor<super::helloworld::HelloRequest, super::helloworld::HelloReply> = ::grpc::method::MethodDescriptor {
            name: "/helloworld.Greeter/SayHello".to_string(),
            client_streaming: false,
            server_streaming: false,
            req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
            resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
        };
        self.grpc_client.call(p, method)
    }
}

// sync server

pub struct GreeterServer {
    server: ::grpc::server_sync::GrpcServer,
}

impl GreeterServer {
    pub fn new<H : Greeter + 'static + Sync + Send>(h: H) -> Self {
        let handler_arc = ::std::sync::Arc::new(h);
        let service_definition = ::grpc::server_sync::ServerServiceDefinitionSync::new(
            vec![
                ::grpc::server_sync::ServerMethodSync::new(
                    ::grpc::method::MethodDescriptor {
                        name: "/helloworld.Greeter/SayHello".to_string(),
                        client_streaming: false,
                        server_streaming: false,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    },
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server_sync::MethodHandlerSyncFn::new(move |p| handler_copy.SayHello(p))
                    },
                ),
            ],
        );
        GreeterServer {
            server: ::grpc::server_sync::GrpcServer::new(service_definition),
        }
    }

    pub fn run(&mut self) {
        self.server.run()
    }
}

// async server

pub struct GreeterAsyncServer {
    grpc_server: ::grpc::server_async::GrpcServerAsync,
}

impl GreeterAsyncServer {
    pub fn new<H : GreeterAsync + 'static + Sync + Send>(port: u16, h: H) -> Self {
        let handler_arc = ::std::sync::Arc::new(h);
        let service_definition = ::grpc::server_async::ServerServiceDefinitionAsync::new(
            vec![
                ::grpc::server_async::ServerMethodAsync::new(
                    ::grpc::method::MethodDescriptor {
                        name: "/helloworld.Greeter/SayHello".to_string(),
                        client_streaming: false,
                        server_streaming: false,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    },
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server_async::MethodHandlerAsyncFn::new(move |p| handler_copy.SayHello(p))
                    },
                ),
            ],
        );
        GreeterAsyncServer {
            grpc_server: ::grpc::server_async::GrpcServerAsync::new(port, service_definition),
        }
    }
}
