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
    fn EmptyCall(&self, o: ::grpc::GrpcRequestOptions, p: super::empty::Empty) -> ::grpc::result::GrpcResult<super::empty::Empty>;

    fn UnaryCall(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::SimpleRequest) -> ::grpc::result::GrpcResult<super::messages::SimpleResponse>;

    fn CacheableUnaryCall(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::SimpleRequest) -> ::grpc::result::GrpcResult<super::messages::SimpleResponse>;

    fn StreamingOutputCall(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::StreamingOutputCallRequest) -> ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallResponse>;

    fn StreamingInputCall(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::iter::GrpcIterator<super::messages::StreamingInputCallRequest>) -> ::grpc::result::GrpcResult<super::messages::StreamingInputCallResponse>;

    fn FullDuplexCall(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallRequest>) -> ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallResponse>;

    fn HalfDuplexCall(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallRequest>) -> ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallResponse>;
}

pub trait TestServiceAsync {
    fn EmptyCall(&self, o: ::grpc::GrpcRequestOptions, p: super::empty::Empty) -> ::grpc::GrpcSingleResponse<super::empty::Empty>;

    fn UnaryCall(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::SimpleRequest) -> ::grpc::GrpcSingleResponse<super::messages::SimpleResponse>;

    fn CacheableUnaryCall(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::SimpleRequest) -> ::grpc::GrpcSingleResponse<super::messages::SimpleResponse>;

    fn StreamingOutputCall(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::StreamingOutputCallRequest) -> ::grpc::GrpcStreamingResponse<super::messages::StreamingOutputCallResponse>;

    fn StreamingInputCall(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingInputCallRequest>) -> ::grpc::GrpcSingleResponse<super::messages::StreamingInputCallResponse>;

    fn FullDuplexCall(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallRequest>) -> ::grpc::GrpcStreamingResponse<super::messages::StreamingOutputCallResponse>;

    fn HalfDuplexCall(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallRequest>) -> ::grpc::GrpcStreamingResponse<super::messages::StreamingOutputCallResponse>;
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
    fn EmptyCall(&self, o: ::grpc::GrpcRequestOptions, p: super::empty::Empty) -> ::grpc::result::GrpcResult<super::empty::Empty> {
        ::grpc::GrpcSingleResponse::wait_drop_metadata(self.async_client.EmptyCall(o, p))
    }

    fn UnaryCall(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::SimpleRequest) -> ::grpc::result::GrpcResult<super::messages::SimpleResponse> {
        ::grpc::GrpcSingleResponse::wait_drop_metadata(self.async_client.UnaryCall(o, p))
    }

    fn CacheableUnaryCall(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::SimpleRequest) -> ::grpc::result::GrpcResult<super::messages::SimpleResponse> {
        ::grpc::GrpcSingleResponse::wait_drop_metadata(self.async_client.CacheableUnaryCall(o, p))
    }

    fn StreamingOutputCall(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::StreamingOutputCallRequest) -> ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallResponse> {
        ::grpc::rt::stream_to_iter(self.async_client.StreamingOutputCall(o, p))
    }

    fn StreamingInputCall(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::iter::GrpcIterator<super::messages::StreamingInputCallRequest>) -> ::grpc::result::GrpcResult<super::messages::StreamingInputCallResponse> {
        let p = ::futures::stream::Stream::boxed(::futures::stream::iter(::std::iter::IntoIterator::into_iter(p)));
        ::grpc::GrpcSingleResponse::wait_drop_metadata(self.async_client.StreamingInputCall(o, p))
    }

    fn FullDuplexCall(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallRequest>) -> ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallResponse> {
        let p = ::futures::stream::Stream::boxed(::futures::stream::iter(::std::iter::IntoIterator::into_iter(p)));
        ::grpc::rt::stream_to_iter(self.async_client.FullDuplexCall(o, p))
    }

    fn HalfDuplexCall(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallRequest>) -> ::grpc::iter::GrpcIterator<super::messages::StreamingOutputCallResponse> {
        let p = ::futures::stream::Stream::boxed(::futures::stream::iter(::std::iter::IntoIterator::into_iter(p)));
        ::grpc::rt::stream_to_iter(self.async_client.HalfDuplexCall(o, p))
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
    pub fn with_client(grpc_client: ::grpc::client::GrpcClient) -> Self {
        TestServiceAsyncClient {
            grpc_client: grpc_client,
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
    }

    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        ::grpc::client::GrpcClient::new(host, port, tls, conf).map(|c| {
            TestServiceAsyncClient::with_client(c)
        })
    }
}

impl TestServiceAsync for TestServiceAsyncClient {
    fn EmptyCall(&self, o: ::grpc::GrpcRequestOptions, p: super::empty::Empty) -> ::grpc::GrpcSingleResponse<super::empty::Empty> {
        self.grpc_client.call_unary(o, p, self.method_EmptyCall.clone())
    }

    fn UnaryCall(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::SimpleRequest) -> ::grpc::GrpcSingleResponse<super::messages::SimpleResponse> {
        self.grpc_client.call_unary(o, p, self.method_UnaryCall.clone())
    }

    fn CacheableUnaryCall(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::SimpleRequest) -> ::grpc::GrpcSingleResponse<super::messages::SimpleResponse> {
        self.grpc_client.call_unary(o, p, self.method_CacheableUnaryCall.clone())
    }

    fn StreamingOutputCall(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::StreamingOutputCallRequest) -> ::grpc::GrpcStreamingResponse<super::messages::StreamingOutputCallResponse> {
        self.grpc_client.call_server_streaming(o, p, self.method_StreamingOutputCall.clone())
    }

    fn StreamingInputCall(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingInputCallRequest>) -> ::grpc::GrpcSingleResponse<super::messages::StreamingInputCallResponse> {
        self.grpc_client.call_client_streaming(o, p, self.method_StreamingInputCall.clone())
    }

    fn FullDuplexCall(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallRequest>) -> ::grpc::GrpcStreamingResponse<super::messages::StreamingOutputCallResponse> {
        self.grpc_client.call_bidi(o, p, self.method_FullDuplexCall.clone())
    }

    fn HalfDuplexCall(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallRequest>) -> ::grpc::GrpcStreamingResponse<super::messages::StreamingOutputCallResponse> {
        self.grpc_client.call_bidi(o, p, self.method_HalfDuplexCall.clone())
    }
}

// server

pub struct TestServiceAsyncServer {
    pub grpc_server: ::grpc::server::GrpcServer,
}

impl ::std::ops::Deref for TestServiceAsyncServer {
    type Target = ::grpc::server::GrpcServer;

    fn deref(&self) -> &Self::Target {
        &self.grpc_server
    }
}

impl TestServiceAsyncServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : TestServiceAsync + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let service_definition = TestServiceAsyncServer::new_service_def(h);
        TestServiceAsyncServer {
            grpc_server: ::grpc::server::GrpcServer::new_plain(addr, conf, service_definition),
        }
    }

    pub fn new_pool<A : ::std::net::ToSocketAddrs, H : TestServiceAsync + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H, cpu_pool: ::futures_cpupool::CpuPool) -> Self {
        let service_definition = TestServiceAsyncServer::new_service_def(h);
        TestServiceAsyncServer {
            grpc_server: ::grpc::server::GrpcServer::new_plain_pool(addr, conf, service_definition, cpu_pool),
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
                        ::grpc::server::MethodHandlerUnary::new(move |o, p| handler_copy.EmptyCall(o, p))
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
                        ::grpc::server::MethodHandlerUnary::new(move |o, p| handler_copy.UnaryCall(o, p))
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
                        ::grpc::server::MethodHandlerUnary::new(move |o, p| handler_copy.CacheableUnaryCall(o, p))
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
                        ::grpc::server::MethodHandlerServerStreaming::new(move |o, p| handler_copy.StreamingOutputCall(o, p))
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
                        ::grpc::server::MethodHandlerClientStreaming::new(move |o, p| handler_copy.StreamingInputCall(o, p))
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
                        ::grpc::server::MethodHandlerBidi::new(move |o, p| handler_copy.FullDuplexCall(o, p))
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
                        ::grpc::server::MethodHandlerBidi::new(move |o, p| handler_copy.HalfDuplexCall(o, p))
                    },
                ),
            ],
        )
    }
}

// interface

pub trait UnimplementedService {
    fn UnimplementedCall(&self, o: ::grpc::GrpcRequestOptions, p: super::empty::Empty) -> ::grpc::result::GrpcResult<super::empty::Empty>;
}

pub trait UnimplementedServiceAsync {
    fn UnimplementedCall(&self, o: ::grpc::GrpcRequestOptions, p: super::empty::Empty) -> ::grpc::GrpcSingleResponse<super::empty::Empty>;
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
    fn UnimplementedCall(&self, o: ::grpc::GrpcRequestOptions, p: super::empty::Empty) -> ::grpc::result::GrpcResult<super::empty::Empty> {
        ::grpc::GrpcSingleResponse::wait_drop_metadata(self.async_client.UnimplementedCall(o, p))
    }
}

// async client

pub struct UnimplementedServiceAsyncClient {
    grpc_client: ::grpc::client::GrpcClient,
    method_UnimplementedCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::empty::Empty, super::empty::Empty>>,
}

impl UnimplementedServiceAsyncClient {
    pub fn with_client(grpc_client: ::grpc::client::GrpcClient) -> Self {
        UnimplementedServiceAsyncClient {
            grpc_client: grpc_client,
            method_UnimplementedCall: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                name: "/grpc.testing.UnimplementedService/UnimplementedCall".to_string(),
                streaming: ::grpc::method::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
            }),
        }
    }

    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        ::grpc::client::GrpcClient::new(host, port, tls, conf).map(|c| {
            UnimplementedServiceAsyncClient::with_client(c)
        })
    }
}

impl UnimplementedServiceAsync for UnimplementedServiceAsyncClient {
    fn UnimplementedCall(&self, o: ::grpc::GrpcRequestOptions, p: super::empty::Empty) -> ::grpc::GrpcSingleResponse<super::empty::Empty> {
        self.grpc_client.call_unary(o, p, self.method_UnimplementedCall.clone())
    }
}

// server

pub struct UnimplementedServiceAsyncServer {
    pub grpc_server: ::grpc::server::GrpcServer,
}

impl ::std::ops::Deref for UnimplementedServiceAsyncServer {
    type Target = ::grpc::server::GrpcServer;

    fn deref(&self) -> &Self::Target {
        &self.grpc_server
    }
}

impl UnimplementedServiceAsyncServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : UnimplementedServiceAsync + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let service_definition = UnimplementedServiceAsyncServer::new_service_def(h);
        UnimplementedServiceAsyncServer {
            grpc_server: ::grpc::server::GrpcServer::new_plain(addr, conf, service_definition),
        }
    }

    pub fn new_pool<A : ::std::net::ToSocketAddrs, H : UnimplementedServiceAsync + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H, cpu_pool: ::futures_cpupool::CpuPool) -> Self {
        let service_definition = UnimplementedServiceAsyncServer::new_service_def(h);
        UnimplementedServiceAsyncServer {
            grpc_server: ::grpc::server::GrpcServer::new_plain_pool(addr, conf, service_definition, cpu_pool),
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
                        ::grpc::server::MethodHandlerUnary::new(move |o, p| handler_copy.UnimplementedCall(o, p))
                    },
                ),
            ],
        )
    }
}

// interface

pub trait ReconnectService {
    fn Start(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::ReconnectParams) -> ::grpc::result::GrpcResult<super::empty::Empty>;

    fn Stop(&self, o: ::grpc::GrpcRequestOptions, p: super::empty::Empty) -> ::grpc::result::GrpcResult<super::messages::ReconnectInfo>;
}

pub trait ReconnectServiceAsync {
    fn Start(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::ReconnectParams) -> ::grpc::GrpcSingleResponse<super::empty::Empty>;

    fn Stop(&self, o: ::grpc::GrpcRequestOptions, p: super::empty::Empty) -> ::grpc::GrpcSingleResponse<super::messages::ReconnectInfo>;
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
    fn Start(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::ReconnectParams) -> ::grpc::result::GrpcResult<super::empty::Empty> {
        ::grpc::GrpcSingleResponse::wait_drop_metadata(self.async_client.Start(o, p))
    }

    fn Stop(&self, o: ::grpc::GrpcRequestOptions, p: super::empty::Empty) -> ::grpc::result::GrpcResult<super::messages::ReconnectInfo> {
        ::grpc::GrpcSingleResponse::wait_drop_metadata(self.async_client.Stop(o, p))
    }
}

// async client

pub struct ReconnectServiceAsyncClient {
    grpc_client: ::grpc::client::GrpcClient,
    method_Start: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::ReconnectParams, super::empty::Empty>>,
    method_Stop: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::empty::Empty, super::messages::ReconnectInfo>>,
}

impl ReconnectServiceAsyncClient {
    pub fn with_client(grpc_client: ::grpc::client::GrpcClient) -> Self {
        ReconnectServiceAsyncClient {
            grpc_client: grpc_client,
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
    }

    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        ::grpc::client::GrpcClient::new(host, port, tls, conf).map(|c| {
            ReconnectServiceAsyncClient::with_client(c)
        })
    }
}

impl ReconnectServiceAsync for ReconnectServiceAsyncClient {
    fn Start(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::ReconnectParams) -> ::grpc::GrpcSingleResponse<super::empty::Empty> {
        self.grpc_client.call_unary(o, p, self.method_Start.clone())
    }

    fn Stop(&self, o: ::grpc::GrpcRequestOptions, p: super::empty::Empty) -> ::grpc::GrpcSingleResponse<super::messages::ReconnectInfo> {
        self.grpc_client.call_unary(o, p, self.method_Stop.clone())
    }
}

// server

pub struct ReconnectServiceAsyncServer {
    pub grpc_server: ::grpc::server::GrpcServer,
}

impl ::std::ops::Deref for ReconnectServiceAsyncServer {
    type Target = ::grpc::server::GrpcServer;

    fn deref(&self) -> &Self::Target {
        &self.grpc_server
    }
}

impl ReconnectServiceAsyncServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : ReconnectServiceAsync + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let service_definition = ReconnectServiceAsyncServer::new_service_def(h);
        ReconnectServiceAsyncServer {
            grpc_server: ::grpc::server::GrpcServer::new_plain(addr, conf, service_definition),
        }
    }

    pub fn new_pool<A : ::std::net::ToSocketAddrs, H : ReconnectServiceAsync + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H, cpu_pool: ::futures_cpupool::CpuPool) -> Self {
        let service_definition = ReconnectServiceAsyncServer::new_service_def(h);
        ReconnectServiceAsyncServer {
            grpc_server: ::grpc::server::GrpcServer::new_plain_pool(addr, conf, service_definition, cpu_pool),
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
                        ::grpc::server::MethodHandlerUnary::new(move |o, p| handler_copy.Start(o, p))
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
                        ::grpc::server::MethodHandlerUnary::new(move |o, p| handler_copy.Stop(o, p))
                    },
                ),
            ],
        )
    }
}
