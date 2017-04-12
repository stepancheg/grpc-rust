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
    fn empty_call(&self, p: super::empty::Empty) -> ::grpc::result::GrpcResult<super::empty::Empty>;

    fn unary_call(&self, p: super::messages::SimpleRequest) -> ::grpc::result::GrpcResult<super::messages::SimpleResponse>;

    fn cacheable_unary_call(&self, p: super::messages::SimpleRequest) -> ::grpc::result::GrpcResult<super::messages::SimpleResponse>;

    fn streaming_output_call(&self, p: super::messages::StreamingOutputCallRequest) -> ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallResponse>;

    fn streaming_input_call(&self, p: ::grpc::iter::GrpcIterator<super::messages::StreamingInputCallRequest>) -> ::grpc::result::GrpcResult<super::messages::StreamingInputCallResponse>;

    fn full_duplex_call(&self, p: ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallRequest>) -> ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallResponse>;

    fn half_duplex_call(&self, p: ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallRequest>) -> ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallResponse>;
}

pub trait TestServiceAsync {
    fn empty_call(&self, p: super::empty::Empty) -> ::grpc::futures_grpc::GrpcFutureSend<super::empty::Empty>;

    fn unary_call(&self, p: super::messages::SimpleRequest) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::SimpleResponse>;

    fn cacheable_unary_call(&self, p: super::messages::SimpleRequest) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::SimpleResponse>;

    fn streaming_output_call(&self, p: super::messages::StreamingOutputCallRequest) -> ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallResponse>;

    fn streaming_input_call(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingInputCallRequest>) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::StreamingInputCallResponse>;

    fn full_duplex_call(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallRequest>) -> ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallResponse>;

    fn half_duplex_call(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallRequest>) -> ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallResponse>;
}

// sync client

pub struct TestServiceClient {
    async_client: TestServiceAsyncClient,
}

impl TestServiceClient {
    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        TestServiceAsyncClient::new(host, port, tls, conf).map(|c| {
            TestServiceClient {
                async_client: c,
            }
        })
    }
}

impl TestService for TestServiceClient {
    fn empty_call(&self, p: super::empty::Empty) -> ::grpc::result::GrpcResult<super::empty::Empty> {
        ::futures::Future::wait(self.async_client.empty_call(p))
    }

    fn unary_call(&self, p: super::messages::SimpleRequest) -> ::grpc::result::GrpcResult<super::messages::SimpleResponse> {
        ::futures::Future::wait(self.async_client.unary_call(p))
    }

    fn cacheable_unary_call(&self, p: super::messages::SimpleRequest) -> ::grpc::result::GrpcResult<super::messages::SimpleResponse> {
        ::futures::Future::wait(self.async_client.cacheable_unary_call(p))
    }

    fn streaming_output_call(&self, p: super::messages::StreamingOutputCallRequest) -> ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallResponse> {
        ::grpc::rt::stream_to_iter(self.async_client.streaming_output_call(p))
    }

    fn streaming_input_call(&self, p: ::grpc::iter::GrpcIterator<super::messages::StreamingInputCallRequest>) -> ::grpc::result::GrpcResult<super::messages::StreamingInputCallResponse> {
        let p = ::futures::stream::Stream::boxed(::futures::stream::iter(::std::iter::IntoIterator::into_iter(p)));
        ::futures::Future::wait(self.async_client.streaming_input_call(p))
    }

    fn full_duplex_call(&self, p: ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallRequest>) -> ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallResponse> {
        let p = ::futures::stream::Stream::boxed(::futures::stream::iter(::std::iter::IntoIterator::into_iter(p)));
        ::grpc::rt::stream_to_iter(self.async_client.full_duplex_call(p))
    }

    fn half_duplex_call(&self, p: ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallRequest>) -> ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallResponse> {
        let p = ::futures::stream::Stream::boxed(::futures::stream::iter(::std::iter::IntoIterator::into_iter(p)));
        ::grpc::rt::stream_to_iter(self.async_client.half_duplex_call(p))
    }
}

// async client

pub struct TestServiceAsyncClient {
    grpc_client: ::grpc::client::GrpcClient,
    method_EmptyCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::empty::Empty, super::empty::Empty>>,
    method_UnaryCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::SimpleRequest, super::messages::SimpleResponse>>,
    method_CacheableUnaryCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::SimpleRequest, super::messages::SimpleResponse>>,
    method_StreamingOutputCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::StreamingOutputCallRequest, super::messages::StreamingOutputCallResponse>>,
    method_StreamingInputCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::StreamingInputCallRequest, super::messages::StreamingInputCallResponse>>,
    method_FullDuplexCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::StreamingOutputCallRequest, super::messages::StreamingOutputCallResponse>>,
    method_HalfDuplexCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::StreamingOutputCallRequest, super::messages::StreamingOutputCallResponse>>,
}

impl TestServiceAsyncClient {
    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        ::grpc::client::GrpcClient::new(host, port, tls, conf).map(|c| {
            TestServiceAsyncClient {
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

impl TestServiceAsync for TestServiceAsyncClient {
    fn empty_call(&self, p: super::empty::Empty) -> ::grpc::futures_grpc::GrpcFutureSend<super::empty::Empty> {
        self.grpc_client.call_unary(p, self.method_EmptyCall.clone())
    }

    fn unary_call(&self, p: super::messages::SimpleRequest) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::SimpleResponse> {
        self.grpc_client.call_unary(p, self.method_UnaryCall.clone())
    }

    fn cacheable_unary_call(&self, p: super::messages::SimpleRequest) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::SimpleResponse> {
        self.grpc_client.call_unary(p, self.method_CacheableUnaryCall.clone())
    }

    fn streaming_output_call(&self, p: super::messages::StreamingOutputCallRequest) -> ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallResponse> {
        self.grpc_client.call_server_streaming(p, self.method_StreamingOutputCall.clone())
    }

    fn streaming_input_call(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingInputCallRequest>) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::StreamingInputCallResponse> {
        self.grpc_client.call_client_streaming(p, self.method_StreamingInputCall.clone())
    }

    fn full_duplex_call(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallRequest>) -> ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallResponse> {
        self.grpc_client.call_bidi(p, self.method_FullDuplexCall.clone())
    }

    fn half_duplex_call(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallRequest>) -> ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallResponse> {
        self.grpc_client.call_bidi(p, self.method_HalfDuplexCall.clone())
    }
}

// sync server

pub struct TestServiceServer {
    async_server: TestServiceAsyncServer,
}

struct TestServiceServerHandlerToAsync {
    handler: ::std::sync::Arc<TestService + Send + Sync>,
    cpupool: ::futures_cpupool::CpuPool,
}

impl TestServiceAsync for TestServiceServerHandlerToAsync {
    fn empty_call(&self, p: super::empty::Empty) -> ::grpc::futures_grpc::GrpcFutureSend<super::empty::Empty> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_unary(&self.cpupool, p, move |p| {
            h.empty_call(p)
        })
    }

    fn unary_call(&self, p: super::messages::SimpleRequest) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::SimpleResponse> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_unary(&self.cpupool, p, move |p| {
            h.unary_call(p)
        })
    }

    fn cacheable_unary_call(&self, p: super::messages::SimpleRequest) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::SimpleResponse> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_unary(&self.cpupool, p, move |p| {
            h.cacheable_unary_call(p)
        })
    }

    fn streaming_output_call(&self, p: super::messages::StreamingOutputCallRequest) -> ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallResponse> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_server_streaming(&self.cpupool, p, move |p| {
            h.streaming_output_call(p)
        })
    }

    fn streaming_input_call(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingInputCallRequest>) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::StreamingInputCallResponse> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_client_streaming(&self.cpupool, p, move |p| {
            h.streaming_input_call(p)
        })
    }

    fn full_duplex_call(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallRequest>) -> ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallResponse> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_bidi(&self.cpupool, p, move |p| {
            h.full_duplex_call(p)
        })
    }

    fn half_duplex_call(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallRequest>) -> ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallResponse> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_bidi(&self.cpupool, p, move |p| {
            h.half_duplex_call(p)
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
            async_server: TestServiceAsyncServer::new(addr, conf, h),
        }
    }
}

// async server

pub struct TestServiceAsyncServer {
    grpc_server: ::grpc::server::GrpcServer,
}

impl TestServiceAsyncServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : TestServiceAsync + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let service_definition = TestServiceAsyncServer::new_service_def(h);
        TestServiceAsyncServer {
            grpc_server: ::grpc::server::GrpcServer::new(addr, conf, service_definition),
        }
    }

    pub fn new_service_def<H : TestServiceAsync + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::server::ServerServiceDefinition {
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
                        ::grpc::server::MethodHandlerUnary::new(move |p| handler_copy.empty_call(p))
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
                        ::grpc::server::MethodHandlerUnary::new(move |p| handler_copy.unary_call(p))
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
                        ::grpc::server::MethodHandlerUnary::new(move |p| handler_copy.cacheable_unary_call(p))
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
                        ::grpc::server::MethodHandlerServerStreaming::new(move |p| handler_copy.streaming_output_call(p))
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
                        ::grpc::server::MethodHandlerClientStreaming::new(move |p| handler_copy.streaming_input_call(p))
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
                        ::grpc::server::MethodHandlerBidi::new(move |p| handler_copy.full_duplex_call(p))
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
                        ::grpc::server::MethodHandlerBidi::new(move |p| handler_copy.half_duplex_call(p))
                    },
                ),
            ],
        )
    }
}

// interface

pub trait UnimplementedService {
    fn unimplemented_call(&self, p: super::empty::Empty) -> ::grpc::result::GrpcResult<super::empty::Empty>;
}

pub trait UnimplementedServiceAsync {
    fn unimplemented_call(&self, p: super::empty::Empty) -> ::grpc::futures_grpc::GrpcFutureSend<super::empty::Empty>;
}

// sync client

pub struct UnimplementedServiceClient {
    async_client: UnimplementedServiceAsyncClient,
}

impl UnimplementedServiceClient {
    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        UnimplementedServiceAsyncClient::new(host, port, tls, conf).map(|c| {
            UnimplementedServiceClient {
                async_client: c,
            }
        })
    }
}

impl UnimplementedService for UnimplementedServiceClient {
    fn unimplemented_call(&self, p: super::empty::Empty) -> ::grpc::result::GrpcResult<super::empty::Empty> {
        ::futures::Future::wait(self.async_client.unimplemented_call(p))
    }
}

// async client

pub struct UnimplementedServiceAsyncClient {
    grpc_client: ::grpc::client::GrpcClient,
    method_UnimplementedCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::empty::Empty, super::empty::Empty>>,
}

impl UnimplementedServiceAsyncClient {
    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        ::grpc::client::GrpcClient::new(host, port, tls, conf).map(|c| {
            UnimplementedServiceAsyncClient {
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

impl UnimplementedServiceAsync for UnimplementedServiceAsyncClient {
    fn unimplemented_call(&self, p: super::empty::Empty) -> ::grpc::futures_grpc::GrpcFutureSend<super::empty::Empty> {
        self.grpc_client.call_unary(p, self.method_UnimplementedCall.clone())
    }
}

// sync server

pub struct UnimplementedServiceServer {
    async_server: UnimplementedServiceAsyncServer,
}

struct UnimplementedServiceServerHandlerToAsync {
    handler: ::std::sync::Arc<UnimplementedService + Send + Sync>,
    cpupool: ::futures_cpupool::CpuPool,
}

impl UnimplementedServiceAsync for UnimplementedServiceServerHandlerToAsync {
    fn unimplemented_call(&self, p: super::empty::Empty) -> ::grpc::futures_grpc::GrpcFutureSend<super::empty::Empty> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_unary(&self.cpupool, p, move |p| {
            h.unimplemented_call(p)
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
            async_server: UnimplementedServiceAsyncServer::new(addr, conf, h),
        }
    }
}

// async server

pub struct UnimplementedServiceAsyncServer {
    grpc_server: ::grpc::server::GrpcServer,
}

impl UnimplementedServiceAsyncServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : UnimplementedServiceAsync + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let service_definition = UnimplementedServiceAsyncServer::new_service_def(h);
        UnimplementedServiceAsyncServer {
            grpc_server: ::grpc::server::GrpcServer::new(addr, conf, service_definition),
        }
    }

    pub fn new_service_def<H : UnimplementedServiceAsync + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::server::ServerServiceDefinition {
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
                        ::grpc::server::MethodHandlerUnary::new(move |p| handler_copy.unimplemented_call(p))
                    },
                ),
            ],
        )
    }
}

// interface

pub trait ReconnectService {
    fn start(&self, p: super::messages::ReconnectParams) -> ::grpc::result::GrpcResult<super::empty::Empty>;

    fn stop(&self, p: super::empty::Empty) -> ::grpc::result::GrpcResult<super::messages::ReconnectInfo>;
}

pub trait ReconnectServiceAsync {
    fn start(&self, p: super::messages::ReconnectParams) -> ::grpc::futures_grpc::GrpcFutureSend<super::empty::Empty>;

    fn stop(&self, p: super::empty::Empty) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::ReconnectInfo>;
}

// sync client

pub struct ReconnectServiceClient {
    async_client: ReconnectServiceAsyncClient,
}

impl ReconnectServiceClient {
    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        ReconnectServiceAsyncClient::new(host, port, tls, conf).map(|c| {
            ReconnectServiceClient {
                async_client: c,
            }
        })
    }
}

impl ReconnectService for ReconnectServiceClient {
    fn start(&self, p: super::messages::ReconnectParams) -> ::grpc::result::GrpcResult<super::empty::Empty> {
        ::futures::Future::wait(self.async_client.start(p))
    }

    fn stop(&self, p: super::empty::Empty) -> ::grpc::result::GrpcResult<super::messages::ReconnectInfo> {
        ::futures::Future::wait(self.async_client.stop(p))
    }
}

// async client

pub struct ReconnectServiceAsyncClient {
    grpc_client: ::grpc::client::GrpcClient,
    method_Start: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::ReconnectParams, super::empty::Empty>>,
    method_Stop: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::empty::Empty, super::messages::ReconnectInfo>>,
}

impl ReconnectServiceAsyncClient {
    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        ::grpc::client::GrpcClient::new(host, port, tls, conf).map(|c| {
            ReconnectServiceAsyncClient {
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

impl ReconnectServiceAsync for ReconnectServiceAsyncClient {
    fn start(&self, p: super::messages::ReconnectParams) -> ::grpc::futures_grpc::GrpcFutureSend<super::empty::Empty> {
        self.grpc_client.call_unary(p, self.method_Start.clone())
    }

    fn stop(&self, p: super::empty::Empty) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::ReconnectInfo> {
        self.grpc_client.call_unary(p, self.method_Stop.clone())
    }
}

// sync server

pub struct ReconnectServiceServer {
    async_server: ReconnectServiceAsyncServer,
}

struct ReconnectServiceServerHandlerToAsync {
    handler: ::std::sync::Arc<ReconnectService + Send + Sync>,
    cpupool: ::futures_cpupool::CpuPool,
}

impl ReconnectServiceAsync for ReconnectServiceServerHandlerToAsync {
    fn start(&self, p: super::messages::ReconnectParams) -> ::grpc::futures_grpc::GrpcFutureSend<super::empty::Empty> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_unary(&self.cpupool, p, move |p| {
            h.start(p)
        })
    }

    fn stop(&self, p: super::empty::Empty) -> ::grpc::futures_grpc::GrpcFutureSend<super::messages::ReconnectInfo> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_unary(&self.cpupool, p, move |p| {
            h.stop(p)
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
            async_server: ReconnectServiceAsyncServer::new(addr, conf, h),
        }
    }
}

// async server

pub struct ReconnectServiceAsyncServer {
    grpc_server: ::grpc::server::GrpcServer,
}

impl ReconnectServiceAsyncServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : ReconnectServiceAsync + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let service_definition = ReconnectServiceAsyncServer::new_service_def(h);
        ReconnectServiceAsyncServer {
            grpc_server: ::grpc::server::GrpcServer::new(addr, conf, service_definition),
        }
    }

    pub fn new_service_def<H : ReconnectServiceAsync + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::server::ServerServiceDefinition {
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
                        ::grpc::server::MethodHandlerUnary::new(move |p| handler_copy.start(p))
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
                        ::grpc::server::MethodHandlerUnary::new(move |p| handler_copy.stop(p))
                    },
                ),
            ],
        )
    }
}
