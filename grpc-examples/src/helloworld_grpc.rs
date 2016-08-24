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
        ::grpc::futuresx::wait2(self.async_client.SayHello(p))
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
    server: ::grpc::server::GrpcServer,
}

impl GreeterServer {
    pub fn new<H : Greeter + 'static + Sync + Send>(h: H) -> GreeterServer {
        let handler_arc = ::std::sync::Arc::new(h);
        let service_definition = ::std::sync::Arc::new(::grpc::method::ServerServiceDefinition::new(
            vec![
                ::grpc::method::ServerMethod::new(
                    ::grpc::method::MethodDescriptor {
                        name: "/helloworld.Greeter/SayHello".to_string(),
                        client_streaming: false,
                        server_streaming: false,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    },
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::method::MethodHandlerFn::new(move |p| handler_copy.SayHello(p))
                    },
                ),
            ],
        ));
        GreeterServer {
            server: ::grpc::server::GrpcServer::new(service_definition),
        }
    }

    pub fn run(&mut self) {
        self.server.run()
    }
}
