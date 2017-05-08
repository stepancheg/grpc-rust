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
    fn EmptyCall(&self, o: ::grpc::GrpcRequestOptions, p: super::empty::Empty) -> ::grpc::GrpcSingleResponse<super::empty::Empty>;

    fn UnaryCall(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::SimpleRequest) -> ::grpc::GrpcSingleResponse<super::messages::SimpleResponse>;

    fn CacheableUnaryCall(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::SimpleRequest) -> ::grpc::GrpcSingleResponse<super::messages::SimpleResponse>;

    fn StreamingOutputCall(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::StreamingOutputCallRequest) -> ::grpc::GrpcStreamingResponse<super::messages::StreamingOutputCallResponse>;

    fn StreamingInputCall(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingInputCallRequest>) -> ::grpc::GrpcSingleResponse<super::messages::StreamingInputCallResponse>;

    fn FullDuplexCall(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallRequest>) -> ::grpc::GrpcStreamingResponse<super::messages::StreamingOutputCallResponse>;

    fn HalfDuplexCall(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::futures_grpc::GrpcStreamSend<super::messages::StreamingOutputCallRequest>) -> ::grpc::GrpcStreamingResponse<super::messages::StreamingOutputCallResponse>;
}

// client

pub struct TestServiceClient {
    grpc_client: ::grpc::client::GrpcClient,
    method_EmptyCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::empty::Empty, super::empty::Empty>>,
    method_UnaryCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::SimpleRequest, super::messages::SimpleResponse>>,
    method_CacheableUnaryCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::SimpleRequest, super::messages::SimpleResponse>>,
    method_StreamingOutputCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::StreamingOutputCallRequest, super::messages::StreamingOutputCallResponse>>,
    method_StreamingInputCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::StreamingInputCallRequest, super::messages::StreamingInputCallResponse>>,
    method_FullDuplexCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::StreamingOutputCallRequest, super::messages::StreamingOutputCallResponse>>,
    method_HalfDuplexCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::StreamingOutputCallRequest, super::messages::StreamingOutputCallResponse>>,
}

impl TestServiceClient {
    pub fn with_client(grpc_client: ::grpc::client::GrpcClient) -> Self {
        TestServiceClient {
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
            TestServiceClient::with_client(c)
        })
    }
}

impl TestService for TestServiceClient {
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

pub struct TestServiceServer {
    pub grpc_server: ::grpc::server::GrpcServer,
}

impl ::std::ops::Deref for TestServiceServer {
    type Target = ::grpc::server::GrpcServer;

    fn deref(&self) -> &Self::Target {
        &self.grpc_server
    }
}

impl TestServiceServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : TestService + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let service_definition = TestServiceServer::new_service_def(h);
        TestServiceServer {
            grpc_server: ::grpc::server::GrpcServer::new_plain(addr, conf, service_definition),
        }
    }

    pub fn new_pool<A : ::std::net::ToSocketAddrs, H : TestService + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H, cpu_pool: ::futures_cpupool::CpuPool) -> Self {
        let service_definition = TestServiceServer::new_service_def(h);
        TestServiceServer {
            grpc_server: ::grpc::server::GrpcServer::new_plain_pool(addr, conf, service_definition, cpu_pool),
        }
    }

    pub fn new_service_def<H : TestService + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::server::ServerServiceDefinition {
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
    fn UnimplementedCall(&self, o: ::grpc::GrpcRequestOptions, p: super::empty::Empty) -> ::grpc::GrpcSingleResponse<super::empty::Empty>;
}

// client

pub struct UnimplementedServiceClient {
    grpc_client: ::grpc::client::GrpcClient,
    method_UnimplementedCall: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::empty::Empty, super::empty::Empty>>,
}

impl UnimplementedServiceClient {
    pub fn with_client(grpc_client: ::grpc::client::GrpcClient) -> Self {
        UnimplementedServiceClient {
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
            UnimplementedServiceClient::with_client(c)
        })
    }
}

impl UnimplementedService for UnimplementedServiceClient {
    fn UnimplementedCall(&self, o: ::grpc::GrpcRequestOptions, p: super::empty::Empty) -> ::grpc::GrpcSingleResponse<super::empty::Empty> {
        self.grpc_client.call_unary(o, p, self.method_UnimplementedCall.clone())
    }
}

// server

pub struct UnimplementedServiceServer {
    pub grpc_server: ::grpc::server::GrpcServer,
}

impl ::std::ops::Deref for UnimplementedServiceServer {
    type Target = ::grpc::server::GrpcServer;

    fn deref(&self) -> &Self::Target {
        &self.grpc_server
    }
}

impl UnimplementedServiceServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : UnimplementedService + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let service_definition = UnimplementedServiceServer::new_service_def(h);
        UnimplementedServiceServer {
            grpc_server: ::grpc::server::GrpcServer::new_plain(addr, conf, service_definition),
        }
    }

    pub fn new_pool<A : ::std::net::ToSocketAddrs, H : UnimplementedService + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H, cpu_pool: ::futures_cpupool::CpuPool) -> Self {
        let service_definition = UnimplementedServiceServer::new_service_def(h);
        UnimplementedServiceServer {
            grpc_server: ::grpc::server::GrpcServer::new_plain_pool(addr, conf, service_definition, cpu_pool),
        }
    }

    pub fn new_service_def<H : UnimplementedService + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::server::ServerServiceDefinition {
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
    fn Start(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::ReconnectParams) -> ::grpc::GrpcSingleResponse<super::empty::Empty>;

    fn Stop(&self, o: ::grpc::GrpcRequestOptions, p: super::empty::Empty) -> ::grpc::GrpcSingleResponse<super::messages::ReconnectInfo>;
}

// client

pub struct ReconnectServiceClient {
    grpc_client: ::grpc::client::GrpcClient,
    method_Start: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::messages::ReconnectParams, super::empty::Empty>>,
    method_Stop: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::empty::Empty, super::messages::ReconnectInfo>>,
}

impl ReconnectServiceClient {
    pub fn with_client(grpc_client: ::grpc::client::GrpcClient) -> Self {
        ReconnectServiceClient {
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
            ReconnectServiceClient::with_client(c)
        })
    }
}

impl ReconnectService for ReconnectServiceClient {
    fn Start(&self, o: ::grpc::GrpcRequestOptions, p: super::messages::ReconnectParams) -> ::grpc::GrpcSingleResponse<super::empty::Empty> {
        self.grpc_client.call_unary(o, p, self.method_Start.clone())
    }

    fn Stop(&self, o: ::grpc::GrpcRequestOptions, p: super::empty::Empty) -> ::grpc::GrpcSingleResponse<super::messages::ReconnectInfo> {
        self.grpc_client.call_unary(o, p, self.method_Stop.clone())
    }
}

// server

pub struct ReconnectServiceServer {
    pub grpc_server: ::grpc::server::GrpcServer,
}

impl ::std::ops::Deref for ReconnectServiceServer {
    type Target = ::grpc::server::GrpcServer;

    fn deref(&self) -> &Self::Target {
        &self.grpc_server
    }
}

impl ReconnectServiceServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : ReconnectService + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let service_definition = ReconnectServiceServer::new_service_def(h);
        ReconnectServiceServer {
            grpc_server: ::grpc::server::GrpcServer::new_plain(addr, conf, service_definition),
        }
    }

    pub fn new_pool<A : ::std::net::ToSocketAddrs, H : ReconnectService + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H, cpu_pool: ::futures_cpupool::CpuPool) -> Self {
        let service_definition = ReconnectServiceServer::new_service_def(h);
        ReconnectServiceServer {
            grpc_server: ::grpc::server::GrpcServer::new_plain_pool(addr, conf, service_definition, cpu_pool),
        }
    }

    pub fn new_service_def<H : ReconnectService + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::server::ServerServiceDefinition {
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
