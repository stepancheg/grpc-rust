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

pub trait TestService {
    fn EmptyCall(&self, p: super::empty::Empty) -> ::grpc::result::GrpcResult<super::empty::Empty>;

    fn UnaryCall(&self, p: super::messages::SimpleRequest) -> ::grpc::result::GrpcResult<super::messages::SimpleResponse>;

    fn CacheableUnaryCall(&self, p: super::messages::SimpleRequest) -> ::grpc::result::GrpcResult<super::messages::SimpleResponse>;

    fn StreamingOutputCall(&self, p: super::messages::StreamingOutputCallRequest) -> ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallResponse>;

    fn StreamingInputCall(&self, p: ::grpc::iter::GrpcIterator<super::messages::StreamingInputCallRequest>) -> ::grpc::result::GrpcResult<super::messages::StreamingInputCallResponse>;

    fn FullDuplexCall(&self, p: ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallRequest>) -> ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallResponse>;

    fn HalfDuplexCall(&self, p: ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallRequest>) -> ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallResponse>;
}

pub trait AsyncTestService {
    fn EmptyCall(&self, p: super::empty::Empty) -> ::grpc::futures_grpc::GrpcFutureSend<super::empty::Empty>;

    fn UnaryCall(&self, p: super::messages::SimpleRequest) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::SimpleResponse>;

    fn CacheableUnaryCall(&self, p: super::messages::SimpleRequest) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::SimpleResponse>;

    fn StreamingOutputCall(&self, p: super::messages::StreamingOutputCallRequest) -> ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallResponse>;

    fn StreamingInputCall(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingInputCallRequest>) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::StreamingInputCallResponse>;

    fn FullDuplexCall(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallRequest>) -> ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallResponse>;

    fn HalfDuplexCall(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallRequest>) -> ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallResponse>;
}

// sync client

pub struct TestServiceClient {
    async_client: AsyncTestServiceClient,
}

impl TestServiceClient {
    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        AsyncTestServiceClient::new(host, port, tls, conf).map(|c| {
            TestServiceClient {
                async_client: c,
            }
        })
    }
}

impl TestService for TestServiceClient {
    fn EmptyCall(&self, p: super::empty::Empty) -> ::grpc::result::GrpcResult<super::empty::Empty> {
        ::futures::Future::wait(self.async_client.EmptyCall(p))
    }

    fn UnaryCall(&self, p: super::messages::SimpleRequest) -> ::grpc::result::GrpcResult<super::messages::SimpleResponse> {
        ::futures::Future::wait(self.async_client.UnaryCall(p))
    }

    fn CacheableUnaryCall(&self, p: super::messages::SimpleRequest) -> ::grpc::result::GrpcResult<super::messages::SimpleResponse> {
        ::futures::Future::wait(self.async_client.CacheableUnaryCall(p))
    }

    fn StreamingOutputCall(&self, p: super::messages::StreamingOutputCallRequest) -> ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallResponse> {
        ::grpc::rt::stream_to_iter(self.async_client.StreamingOutputCall(p))
    }

    fn StreamingInputCall(&self, p: ::grpc::iter::GrpcIterator<super::messages::StreamingInputCallRequest>) -> ::grpc::result::GrpcResult<super::messages::StreamingInputCallResponse> {
        let p = ::futures::stream::Stream::boxed(::futures::stream::iter(::std::iter::IntoIterator::into_iter(p)));
        ::futures::Future::wait(self.async_client.StreamingInputCall(p))
    }

    fn FullDuplexCall(&self, p: ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallRequest>) -> ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallResponse> {
        let p = ::futures::stream::Stream::boxed(::futures::stream::iter(::std::iter::IntoIterator::into_iter(p)));
        ::grpc::rt::stream_to_iter(self.async_client.FullDuplexCall(p))
    }

    fn HalfDuplexCall(&self, p: ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallRequest>) -> ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallResponse> {
        let p = ::futures::stream::Stream::boxed(::futures::stream::iter(::std::iter::IntoIterator::into_iter(p)));
        ::grpc::rt::stream_to_iter(self.async_client.HalfDuplexCall(p))
    }
}

// async client

pub struct AsyncTestServiceClient {
    grpc_client: ::grpc::client::GrpcClient,
    method_EmptyCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::empty::Empty, super::empty::Empty>>,
    method_UnaryCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::SimpleRequest, super::messages::SimpleResponse>>,
    method_CacheableUnaryCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::SimpleRequest, super::messages::SimpleResponse>>,
    method_StreamingOutputCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::StreamingOutputCallRequest, super::messages::StreamingOutputCallResponse>>,
    method_StreamingInputCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::StreamingInputCallRequest, super::messages::StreamingInputCallResponse>>,
    method_FullDuplexCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::StreamingOutputCallRequest, super::messages::StreamingOutputCallResponse>>,
    method_HalfDuplexCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::StreamingOutputCallRequest, super::messages::StreamingOutputCallResponse>>,
}

impl AsyncTestServiceClient {
    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        ::grpc::client::GrpcClient::new(host, port, tls, conf).map(|c| {
            AsyncTestServiceClient {
                grpc_client: c,
                method_EmptyCall: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                    name: "/grpc.testing.TestService/EmptyCall".to_string(),
                    streaming: ::grpc::method::GrpcStreaming::Unary,
                    req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                }),
                method_UnaryCall: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                    name: "/grpc.testing.TestService/UnaryCall".to_string(),
                    streaming: ::grpc::method::GrpcStreaming::Unary,
                    req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                }),
                method_CacheableUnaryCall: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                    name: "/grpc.testing.TestService/CacheableUnaryCall".to_string(),
                    streaming: ::grpc::method::GrpcStreaming::Unary,
                    req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                }),
                method_StreamingOutputCall: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                    name: "/grpc.testing.TestService/StreamingOutputCall".to_string(),
                    streaming: ::grpc::method::GrpcStreaming::ServerStreaming,
                    req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                }),
                method_StreamingInputCall: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                    name: "/grpc.testing.TestService/StreamingInputCall".to_string(),
                    streaming: ::grpc::method::GrpcStreaming::ClientStreaming,
                    req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                }),
                method_FullDuplexCall: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                    name: "/grpc.testing.TestService/FullDuplexCall".to_string(),
                    streaming: ::grpc::method::GrpcStreaming::Bidi,
                    req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                }),
                method_HalfDuplexCall: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                    name: "/grpc.testing.TestService/HalfDuplexCall".to_string(),
                    streaming: ::grpc::method::GrpcStreaming::Bidi,
                    req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                }),
            }
        })
    }
}

impl AsyncTestService for AsyncTestServiceClient {
    fn EmptyCall(&self, p: super::empty::Empty) -> ::grpc::futures_grpc::GrpcFutureSend<super::empty::Empty> {
        self.grpc_client.call_unary(p, self.method_EmptyCall.clone())
    }

    fn UnaryCall(&self, p: super::messages::SimpleRequest) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::SimpleResponse> {
        self.grpc_client.call_unary(p, self.method_UnaryCall.clone())
    }

    fn CacheableUnaryCall(&self, p: super::messages::SimpleRequest) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::SimpleResponse> {
        self.grpc_client.call_unary(p, self.method_CacheableUnaryCall.clone())
    }

    fn StreamingOutputCall(&self, p: super::messages::StreamingOutputCallRequest) -> ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallResponse> {
        self.grpc_client.call_server_streaming(p, self.method_StreamingOutputCall.clone())
    }

    fn StreamingInputCall(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingInputCallRequest>) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::StreamingInputCallResponse> {
        self.grpc_client.call_client_streaming(p, self.method_StreamingInputCall.clone())
    }

    fn FullDuplexCall(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallRequest>) -> ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallResponse> {
        self.grpc_client.call_bidi(p, self.method_FullDuplexCall.clone())
    }

    fn HalfDuplexCall(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallRequest>) -> ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallResponse> {
        self.grpc_client.call_bidi(p, self.method_HalfDuplexCall.clone())
    }
}

// sync server

pub struct TestServiceServer {
    async_server: AsyncTestServiceServer,
}

struct TestServiceServerHandlerToAsync {
    handler: ::std::sync::Arc<TestService + Send + Sync>,
    cpupool: ::futures_cpupool::CpuPool,
}

impl AsyncTestService for TestServiceServerHandlerToAsync {
    fn EmptyCall(&self, p: super::empty::Empty) -> ::grpc::futures_grpc::GrpcFutureSend<super::empty::Empty> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_unary(&self.cpupool, p, move |p| {
            h.EmptyCall(p)
        })
    }

    fn UnaryCall(&self, p: super::messages::SimpleRequest) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::SimpleResponse> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_unary(&self.cpupool, p, move |p| {
            h.UnaryCall(p)
        })
    }

    fn CacheableUnaryCall(&self, p: super::messages::SimpleRequest) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::SimpleResponse> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_unary(&self.cpupool, p, move |p| {
            h.CacheableUnaryCall(p)
        })
    }

    fn StreamingOutputCall(&self, p: super::messages::StreamingOutputCallRequest) -> ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallResponse> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_server_streaming(&self.cpupool, p, move |p| {
            h.StreamingOutputCall(p)
        })
    }

    fn StreamingInputCall(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingInputCallRequest>) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::StreamingInputCallResponse> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_client_streaming(&self.cpupool, p, move |p| {
            h.StreamingInputCall(p)
        })
    }

    fn FullDuplexCall(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallRequest>) -> ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallResponse> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_bidi(&self.cpupool, p, move |p| {
            h.FullDuplexCall(p)
        })
    }

    fn HalfDuplexCall(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallRequest>) -> ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallResponse> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_bidi(&self.cpupool, p, move |p| {
            h.HalfDuplexCall(p)
        })
    }
}

impl TestServiceServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : TestService + Send + Sync + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let h = TestServiceServerHandlerToAsync {
            cpupool: ::futures_cpupool::CpuPool::new_num_cpus(),
            handler: ::std::sync::Arc::new(h),
        };
        TestServiceServer {
            async_server: AsyncTestServiceServer::new(addr, conf, h),
        }
    }
}

// async server

pub struct AsyncTestServiceServer {
    grpc_server: ::grpc::server::GrpcServer,
}

impl AsyncTestServiceServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : AsyncTestService + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let service_definition = AsyncTestServiceServer::new_service_def(h);
        AsyncTestServiceServer {
            grpc_server: ::grpc::server::GrpcServer::new(addr, conf, service_definition),
        }
    }

    pub fn new_service_def<H : AsyncTestService + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::server::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::server::ServerServiceDefinition::new(
            vec![
                ::grpc::server::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                        name: "/grpc.testing.TestService/EmptyCall".to_string(),
                        streaming: ::grpc::method::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerUnary::new(move |p| handler_copy.EmptyCall(p))
                    },
                ),
                ::grpc::server::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                        name: "/grpc.testing.TestService/UnaryCall".to_string(),
                        streaming: ::grpc::method::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerUnary::new(move |p| handler_copy.UnaryCall(p))
                    },
                ),
                ::grpc::server::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                        name: "/grpc.testing.TestService/CacheableUnaryCall".to_string(),
                        streaming: ::grpc::method::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerUnary::new(move |p| handler_copy.CacheableUnaryCall(p))
                    },
                ),
                ::grpc::server::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                        name: "/grpc.testing.TestService/StreamingOutputCall".to_string(),
                        streaming: ::grpc::method::GrpcStreaming::ServerStreaming,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerServerStreaming::new(move |p| handler_copy.StreamingOutputCall(p))
                    },
                ),
                ::grpc::server::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                        name: "/grpc.testing.TestService/StreamingInputCall".to_string(),
                        streaming: ::grpc::method::GrpcStreaming::ClientStreaming,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerClientStreaming::new(move |p| handler_copy.StreamingInputCall(p))
                    },
                ),
                ::grpc::server::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                        name: "/grpc.testing.TestService/FullDuplexCall".to_string(),
                        streaming: ::grpc::method::GrpcStreaming::Bidi,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerBidi::new(move |p| handler_copy.FullDuplexCall(p))
                    },
                ),
                ::grpc::server::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                        name: "/grpc.testing.TestService/HalfDuplexCall".to_string(),
                        streaming: ::grpc::method::GrpcStreaming::Bidi,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerBidi::new(move |p| handler_copy.HalfDuplexCall(p))
                    },
                ),
            ],
        )
    }
}

// interface

pub trait UnimplementedService {
    fn UnimplementedCall(&self, p: super::empty::Empty) -> ::grpc::result::GrpcResult<super::empty::Empty>;
}

pub trait AsyncUnimplementedService {
    fn UnimplementedCall(&self, p: super::empty::Empty) -> ::grpc::futures_grpc::GrpcFutureSend<super::empty::Empty>;
}

// sync client

pub struct UnimplementedServiceClient {
    async_client: AsyncUnimplementedServiceClient,
}

impl UnimplementedServiceClient {
    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        AsyncUnimplementedServiceClient::new(host, port, tls, conf).map(|c| {
            UnimplementedServiceClient {
                async_client: c,
            }
        })
    }
}

impl UnimplementedService for UnimplementedServiceClient {
    fn UnimplementedCall(&self, p: super::empty::Empty) -> ::grpc::result::GrpcResult<super::empty::Empty> {
        ::futures::Future::wait(self.async_client.UnimplementedCall(p))
    }
}

// async client

pub struct AsyncUnimplementedServiceClient {
    grpc_client: ::grpc::client::GrpcClient,
    method_UnimplementedCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::empty::Empty, super::empty::Empty>>,
}

impl AsyncUnimplementedServiceClient {
    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        ::grpc::client::GrpcClient::new(host, port, tls, conf).map(|c| {
            AsyncUnimplementedServiceClient {
                grpc_client: c,
                method_UnimplementedCall: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                    name: "/grpc.testing.UnimplementedService/UnimplementedCall".to_string(),
                    streaming: ::grpc::method::GrpcStreaming::Unary,
                    req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                }),
            }
        })
    }
}

impl AsyncUnimplementedService for AsyncUnimplementedServiceClient {
    fn UnimplementedCall(&self, p: super::empty::Empty) -> ::grpc::futures_grpc::GrpcFutureSend<super::empty::Empty> {
        self.grpc_client.call_unary(p, self.method_UnimplementedCall.clone())
    }
}

// sync server

pub struct UnimplementedServiceServer {
    async_server: AsyncUnimplementedServiceServer,
}

struct UnimplementedServiceServerHandlerToAsync {
    handler: ::std::sync::Arc<UnimplementedService + Send + Sync>,
    cpupool: ::futures_cpupool::CpuPool,
}

impl AsyncUnimplementedService for UnimplementedServiceServerHandlerToAsync {
    fn UnimplementedCall(&self, p: super::empty::Empty) -> ::grpc::futures_grpc::GrpcFutureSend<super::empty::Empty> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_unary(&self.cpupool, p, move |p| {
            h.UnimplementedCall(p)
        })
    }
}

impl UnimplementedServiceServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : UnimplementedService + Send + Sync + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let h = UnimplementedServiceServerHandlerToAsync {
            cpupool: ::futures_cpupool::CpuPool::new_num_cpus(),
            handler: ::std::sync::Arc::new(h),
        };
        UnimplementedServiceServer {
            async_server: AsyncUnimplementedServiceServer::new(addr, conf, h),
        }
    }
}

// async server

pub struct AsyncUnimplementedServiceServer {
    grpc_server: ::grpc::server::GrpcServer,
}

impl AsyncUnimplementedServiceServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : AsyncUnimplementedService + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let service_definition = AsyncUnimplementedServiceServer::new_service_def(h);
        AsyncUnimplementedServiceServer {
            grpc_server: ::grpc::server::GrpcServer::new(addr, conf, service_definition),
        }
    }

    pub fn new_service_def<H : AsyncUnimplementedService + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::server::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::server::ServerServiceDefinition::new(
            vec![
                ::grpc::server::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                        name: "/grpc.testing.UnimplementedService/UnimplementedCall".to_string(),
                        streaming: ::grpc::method::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerUnary::new(move |p| handler_copy.UnimplementedCall(p))
                    },
                ),
            ],
        )
    }
}

// interface

pub trait ReconnectService {
    fn Start(&self, p: super::messages::ReconnectParams) -> ::grpc::result::GrpcResult<super::empty::Empty>;

    fn Stop(&self, p: super::empty::Empty) -> ::grpc::result::GrpcResult<super::messages::ReconnectInfo>;
}

pub trait AsyncReconnectService {
    fn Start(&self, p: super::messages::ReconnectParams) -> ::grpc::futures_grpc::GrpcFutureSend<super::empty::Empty>;

    fn Stop(&self, p: super::empty::Empty) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::ReconnectInfo>;
}

// sync client

pub struct ReconnectServiceClient {
    async_client: AsyncReconnectServiceClient,
}

impl ReconnectServiceClient {
    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        AsyncReconnectServiceClient::new(host, port, tls, conf).map(|c| {
            ReconnectServiceClient {
                async_client: c,
            }
        })
    }
}

impl ReconnectService for ReconnectServiceClient {
    fn Start(&self, p: super::messages::ReconnectParams) -> ::grpc::result::GrpcResult<super::empty::Empty> {
        ::futures::Future::wait(self.async_client.Start(p))
    }

    fn Stop(&self, p: super::empty::Empty) -> ::grpc::result::GrpcResult<super::messages::ReconnectInfo> {
        ::futures::Future::wait(self.async_client.Stop(p))
    }
}

// async client

pub struct AsyncReconnectServiceClient {
    grpc_client: ::grpc::client::GrpcClient,
    method_Start: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::ReconnectParams, super::empty::Empty>>,
    method_Stop: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::empty::Empty, super::messages::ReconnectInfo>>,
}

impl AsyncReconnectServiceClient {
    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        ::grpc::client::GrpcClient::new(host, port, tls, conf).map(|c| {
            AsyncReconnectServiceClient {
                grpc_client: c,
                method_Start: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                    name: "/grpc.testing.ReconnectService/Start".to_string(),
                    streaming: ::grpc::method::GrpcStreaming::Unary,
                    req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                }),
                method_Stop: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                    name: "/grpc.testing.ReconnectService/Stop".to_string(),
                    streaming: ::grpc::method::GrpcStreaming::Unary,
                    req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                }),
            }
        })
    }
}

impl AsyncReconnectService for AsyncReconnectServiceClient {
    fn Start(&self, p: super::messages::ReconnectParams) -> ::grpc::futures_grpc::GrpcFutureSend<super::empty::Empty> {
        self.grpc_client.call_unary(p, self.method_Start.clone())
    }

    fn Stop(&self, p: super::empty::Empty) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::ReconnectInfo> {
        self.grpc_client.call_unary(p, self.method_Stop.clone())
    }
}

// sync server

pub struct ReconnectServiceServer {
    async_server: AsyncReconnectServiceServer,
}

struct ReconnectServiceServerHandlerToAsync {
    handler: ::std::sync::Arc<ReconnectService + Send + Sync>,
    cpupool: ::futures_cpupool::CpuPool,
}

impl AsyncReconnectService for ReconnectServiceServerHandlerToAsync {
    fn Start(&self, p: super::messages::ReconnectParams) -> ::grpc::futures_grpc::GrpcFutureSend<super::empty::Empty> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_unary(&self.cpupool, p, move |p| {
            h.Start(p)
        })
    }

    fn Stop(&self, p: super::empty::Empty) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::ReconnectInfo> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_unary(&self.cpupool, p, move |p| {
            h.Stop(p)
        })
    }
}

impl ReconnectServiceServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : ReconnectService + Send + Sync + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let h = ReconnectServiceServerHandlerToAsync {
            cpupool: ::futures_cpupool::CpuPool::new_num_cpus(),
            handler: ::std::sync::Arc::new(h),
        };
        ReconnectServiceServer {
            async_server: AsyncReconnectServiceServer::new(addr, conf, h),
        }
    }
}

// async server

pub struct AsyncReconnectServiceServer {
    grpc_server: ::grpc::server::GrpcServer,
}

impl AsyncReconnectServiceServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : AsyncReconnectService + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let service_definition = AsyncReconnectServiceServer::new_service_def(h);
        AsyncReconnectServiceServer {
            grpc_server: ::grpc::server::GrpcServer::new(addr, conf, service_definition),
        }
    }

    pub fn new_service_def<H : AsyncReconnectService + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::server::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::server::ServerServiceDefinition::new(
            vec![
                ::grpc::server::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                        name: "/grpc.testing.ReconnectService/Start".to_string(),
                        streaming: ::grpc::method::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerUnary::new(move |p| handler_copy.Start(p))
                    },
                ),
                ::grpc::server::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                        name: "/grpc.testing.ReconnectService/Stop".to_string(),
                        streaming: ::grpc::method::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerUnary::new(move |p| handler_copy.Stop(p))
                    },
                ),
            ],
        )
    }
}
