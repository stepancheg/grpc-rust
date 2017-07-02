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

pub trait LongTests {
    fn echo(&self, o: ::grpc::RequestOptions, p: super::long_tests_pb::EchoRequest) -> ::grpc::SingleResponse<super::long_tests_pb::EchoResponse>;

    fn char_count(&self, o: ::grpc::RequestOptions, p: ::grpc::StreamingRequest<super::long_tests_pb::CharCountRequest>) -> ::grpc::SingleResponse<super::long_tests_pb::CharCountResponse>;

    fn random_strings(&self, o: ::grpc::RequestOptions, p: super::long_tests_pb::RandomStringsRequest) -> ::grpc::StreamingResponse<super::long_tests_pb::RandomStringsResponse>;
}

// client

pub struct LongTestsClient {
    grpc_client: ::grpc::Client,
    method_echo: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::long_tests_pb::EchoRequest, super::long_tests_pb::EchoResponse>>,
    method_char_count: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::long_tests_pb::CharCountRequest, super::long_tests_pb::CharCountResponse>>,
    method_random_strings: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::long_tests_pb::RandomStringsRequest, super::long_tests_pb::RandomStringsResponse>>,
}

impl LongTestsClient {
    pub fn with_client(grpc_client: ::grpc::Client) -> Self {
        LongTestsClient {
            grpc_client: grpc_client,
            method_echo: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/LongTests/echo".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_char_count: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/LongTests/char_count".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::ClientStreaming,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_random_strings: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/LongTests/random_strings".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::ServerStreaming,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }

    pub fn new_plain(host: &str, port: u16, conf: ::grpc::ClientConf) -> ::grpc::Result<Self> {
        ::grpc::Client::new_plain(host, port, conf).map(|c| {
            LongTestsClient::with_client(c)
        })
    }
    pub fn new_tls<C : ::tls_api::TlsConnector>(host: &str, port: u16, conf: ::grpc::ClientConf) -> ::grpc::Result<Self> {
        ::grpc::Client::new_tls::<C>(host, port, conf).map(|c| {
            LongTestsClient::with_client(c)
        })
    }
}

impl LongTests for LongTestsClient {
    fn echo(&self, o: ::grpc::RequestOptions, p: super::long_tests_pb::EchoRequest) -> ::grpc::SingleResponse<super::long_tests_pb::EchoResponse> {
        self.grpc_client.call_unary(o, p, self.method_echo.clone())
    }

    fn char_count(&self, o: ::grpc::RequestOptions, p: ::grpc::StreamingRequest<super::long_tests_pb::CharCountRequest>) -> ::grpc::SingleResponse<super::long_tests_pb::CharCountResponse> {
        self.grpc_client.call_client_streaming(o, p, self.method_char_count.clone())
    }

    fn random_strings(&self, o: ::grpc::RequestOptions, p: super::long_tests_pb::RandomStringsRequest) -> ::grpc::StreamingResponse<super::long_tests_pb::RandomStringsResponse> {
        self.grpc_client.call_server_streaming(o, p, self.method_random_strings.clone())
    }
}

// server

pub struct LongTestsServer {
    pub grpc_server: ::grpc::Server,
}

impl ::std::ops::Deref for LongTestsServer {
    type Target = ::grpc::Server;

    fn deref(&self) -> &Self::Target {
        &self.grpc_server
    }
}

impl LongTestsServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : LongTests + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::ServerConf, h: H) -> ::grpc::Result<Self> {
        let service_definition = LongTestsServer::new_service_def(h);
        Ok(LongTestsServer {
            grpc_server: ::grpc::Server::new_plain(addr, conf, service_definition)?,
        })
    }

    pub fn new_pool<A : ::std::net::ToSocketAddrs, H : LongTests + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::ServerConf, h: H, cpu_pool: ::futures_cpupool::CpuPool) -> ::grpc::Result<Self> {
        let service_definition = LongTestsServer::new_service_def(h);
        Ok(LongTestsServer {
            grpc_server: ::grpc::Server::new_plain_pool(addr, conf, service_definition, cpu_pool)?,
        })
    }

    pub fn new_service_def<H : LongTests + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/LongTests",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/LongTests/echo".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.echo(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/LongTests/char_count".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::ClientStreaming,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerClientStreaming::new(move |o, p| handler_copy.char_count(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/LongTests/random_strings".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::ServerStreaming,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerServerStreaming::new(move |o, p| handler_copy.random_strings(o, p))
                    },
                ),
            ],
        )
    }
}
