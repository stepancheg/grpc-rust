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

pub trait LongTests {
    fn echo(&self, p: super::long_tests_pb::EchoRequest) -> ::grpc::result::GrpcResult<super::long_tests_pb::EchoResponse>;

    fn char_count(&self, p: ::grpc::iter::GrpcIterator<super::long_tests_pb::CharCountRequest>) -> ::grpc::result::GrpcResult<super::long_tests_pb::CharCountResponse>;

    fn random_strings(&self, p: super::long_tests_pb::RandomStringsRequest) -> ::grpc::iter::GrpcIterator<super::long_tests_pb::RandomStringsResponse>;
}

pub trait LongTestsAsync {
    fn echo(&self, p: super::long_tests_pb::EchoRequest) -> ::grpc::futures_grpc::GrpcFutureSend<super::long_tests_pb::EchoResponse>;

    fn char_count(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::long_tests_pb::CharCountRequest>) -> ::grpc::futures_grpc::GrpcFutureSend<super::long_tests_pb::CharCountResponse>;

    fn random_strings(&self, p: super::long_tests_pb::RandomStringsRequest) -> ::grpc::futures_grpc::GrpcStreamSend<super::long_tests_pb::RandomStringsResponse>;
}

// sync client

pub struct LongTestsClient {
    async_client: LongTestsAsyncClient,
}

impl LongTestsClient {
    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        LongTestsAsyncClient::new(host, port, tls, conf).map(|c| {
            LongTestsClient {
                async_client: c,
            }
        })
    }
}

impl LongTests for LongTestsClient {
    fn echo(&self, p: super::long_tests_pb::EchoRequest) -> ::grpc::result::GrpcResult<super::long_tests_pb::EchoResponse> {
        ::futures::Future::wait(self.async_client.echo(p))
    }

    fn char_count(&self, p: ::grpc::iter::GrpcIterator<super::long_tests_pb::CharCountRequest>) -> ::grpc::result::GrpcResult<super::long_tests_pb::CharCountResponse> {
        let p = ::futures::stream::Stream::boxed(::futures::stream::iter(::std::iter::IntoIterator::into_iter(p)));
        ::futures::Future::wait(self.async_client.char_count(p))
    }

    fn random_strings(&self, p: super::long_tests_pb::RandomStringsRequest) -> ::grpc::iter::GrpcIterator<super::long_tests_pb::RandomStringsResponse> {
        ::grpc::rt::stream_to_iter(self.async_client.random_strings(p))
    }
}

// async client

pub struct LongTestsAsyncClient {
    grpc_client: ::grpc::client::GrpcClient,
    method_echo: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::long_tests_pb::EchoRequest, super::long_tests_pb::EchoResponse>>,
    method_char_count: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::long_tests_pb::CharCountRequest, super::long_tests_pb::CharCountResponse>>,
    method_random_strings: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::long_tests_pb::RandomStringsRequest, super::long_tests_pb::RandomStringsResponse>>,
}

impl LongTestsAsyncClient {
    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        ::grpc::client::GrpcClient::new(host, port, tls, conf).map(|c| {
            LongTestsAsyncClient {
                grpc_client: c,
                method_echo: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                    name: "/LongTests/echo".to_string(),
                    streaming: ::grpc::method::GrpcStreaming::Unary,
                    req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                }),
                method_char_count: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                    name: "/LongTests/char_count".to_string(),
                    streaming: ::grpc::method::GrpcStreaming::ClientStreaming,
                    req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                }),
                method_random_strings: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                    name: "/LongTests/random_strings".to_string(),
                    streaming: ::grpc::method::GrpcStreaming::ServerStreaming,
                    req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                }),
            }
        })
    }
}

impl LongTestsAsync for LongTestsAsyncClient {
    fn echo(&self, p: super::long_tests_pb::EchoRequest) -> ::grpc::futures_grpc::GrpcFutureSend<super::long_tests_pb::EchoResponse> {
        self.grpc_client.call_unary(p, self.method_echo.clone())
    }

    fn char_count(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::long_tests_pb::CharCountRequest>) -> ::grpc::futures_grpc::GrpcFutureSend<super::long_tests_pb::CharCountResponse> {
        self.grpc_client.call_client_streaming(p, self.method_char_count.clone())
    }

    fn random_strings(&self, p: super::long_tests_pb::RandomStringsRequest) -> ::grpc::futures_grpc::GrpcStreamSend<super::long_tests_pb::RandomStringsResponse> {
        self.grpc_client.call_server_streaming(p, self.method_random_strings.clone())
    }
}

// sync server

pub struct LongTestsServer {
    async_server: LongTestsAsyncServer,
}

struct LongTestsServerHandlerToAsync {
    handler: ::std::sync::Arc<LongTests + Send + Sync>,
    cpupool: ::futures_cpupool::CpuPool,
}

impl LongTestsAsync for LongTestsServerHandlerToAsync {
    fn echo(&self, p: super::long_tests_pb::EchoRequest) -> ::grpc::futures_grpc::GrpcFutureSend<super::long_tests_pb::EchoResponse> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_unary(&self.cpupool, p, move |p| {
            h.echo(p)
        })
    }

    fn char_count(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::long_tests_pb::CharCountRequest>) -> ::grpc::futures_grpc::GrpcFutureSend<super::long_tests_pb::CharCountResponse> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_client_streaming(&self.cpupool, p, move |p| {
            h.char_count(p)
        })
    }

    fn random_strings(&self, p: super::long_tests_pb::RandomStringsRequest) -> ::grpc::futures_grpc::GrpcStreamSend<super::long_tests_pb::RandomStringsResponse> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_server_streaming(&self.cpupool, p, move |p| {
            h.random_strings(p)
        })
    }
}

impl LongTestsServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : LongTests + Send + Sync + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let h = LongTestsServerHandlerToAsync {
            cpupool: ::futures_cpupool::CpuPool::new_num_cpus(),
            handler: ::std::sync::Arc::new(h),
        };
        LongTestsServer {
            async_server: LongTestsAsyncServer::new(addr, conf, h),
        }
    }
}

// async server

pub struct LongTestsAsyncServer {
    grpc_server: ::grpc::server::GrpcServer,
}

impl LongTestsAsyncServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : LongTestsAsync + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let service_definition = LongTestsAsyncServer::new_service_def(h);
        LongTestsAsyncServer {
            grpc_server: ::grpc::server::GrpcServer::new(addr, conf, service_definition),
        }
    }

    pub fn new_service_def<H : LongTestsAsync + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::server::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::server::ServerServiceDefinition::new(
            vec![
                ::grpc::server::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                        name: "/LongTests/echo".to_string(),
                        streaming: ::grpc::method::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerUnary::new(move |p| handler_copy.echo(p))
                    },
                ),
                ::grpc::server::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                        name: "/LongTests/char_count".to_string(),
                        streaming: ::grpc::method::GrpcStreaming::ClientStreaming,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerClientStreaming::new(move |p| handler_copy.char_count(p))
                    },
                ),
                ::grpc::server::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                        name: "/LongTests/random_strings".to_string(),
                        streaming: ::grpc::method::GrpcStreaming::ServerStreaming,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerServerStreaming::new(move |p| handler_copy.random_strings(p))
                    },
                ),
            ],
        )
    }
}
