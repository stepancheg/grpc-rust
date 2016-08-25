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
    method_SayHello: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::helloworld::HelloRequest, super::helloworld::HelloReply>>,
}

impl GreeterAsyncClient {
    pub fn new(host: &str, port: u16) -> Self {
        GreeterAsyncClient {
            grpc_client: ::grpc::client::GrpcClient::new(host, port),
            method_SayHello: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                name: "/helloworld.Greeter/SayHello".to_string(),
                client_streaming: false,
                server_streaming: false,
                req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
            }),
        }
    }
}

impl GreeterAsync for GreeterAsyncClient {
    fn SayHello(&self, p: super::helloworld::HelloRequest) -> ::grpc::futures_grpc::GrpcFuture<super::helloworld::HelloReply> {
        self.grpc_client.call(p, self.method_SayHello.clone())
    }
}

// sync server

pub struct GreeterServer {
    async_server: GreeterAsyncServer,
}

struct GreeterServerHandlerToAsync {
    handler: ::std::sync::Arc<Greeter + Send + Sync>,
    cpupool: ::futures_cpupool::CpuPool,
}

impl GreeterAsync for GreeterServerHandlerToAsync {
    fn SayHello(&self, p: super::helloworld::HelloRequest) -> ::grpc::futures_grpc::GrpcFuture<super::helloworld::HelloReply> {
        let h = self.handler.clone();
        ::futures::Future::boxed(::futures::Future::map_err(self.cpupool.execute(move || {
            h.SayHello(p).unwrap()
        }), |_| ::grpc::error::GrpcError::Other("cpupool")))
    }
}

impl GreeterServer {
    pub fn new<H : Greeter + Send + Sync + 'static>(port: u16, h: H) -> Self {
        let h = GreeterServerHandlerToAsync {
            cpupool: ::futures_cpupool::CpuPool::new_num_cpus(),
            handler: ::std::sync::Arc::new(h),
        };
        GreeterServer {
            async_server: GreeterAsyncServer::new(port, h),
        }
    }
}

// async server

pub struct GreeterAsyncServer {
    grpc_server: ::grpc::server::GrpcServer,
}

impl GreeterAsyncServer {
    pub fn new<H : GreeterAsync + 'static + Sync + Send + 'static>(port: u16, h: H) -> Self {
        let handler_arc = ::std::sync::Arc::new(h);
        let service_definition = ::grpc::server::ServerServiceDefinition::new(
            vec![
                ::grpc::server::ServerMethod::new(
                    ::grpc::method::MethodDescriptor {
                        name: "/helloworld.Greeter/SayHello".to_string(),
                        client_streaming: false,
                        server_streaming: false,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    },
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerFn::new(move |p| handler_copy.SayHello(p))
                    },
                ),
            ],
        );
        GreeterAsyncServer {
            grpc_server: ::grpc::server::GrpcServer::new(port, service_definition),
        }
    }
}
