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

pub trait RouteGuide {
    fn GetFeature(&self, o: ::grpc::GrpcRequestOptions, p: super::route_guide::Point) -> ::grpc::GrpcSingleResponse<super::route_guide::Feature>;

    fn ListFeatures(&self, o: ::grpc::GrpcRequestOptions, p: super::route_guide::Rectangle) -> ::grpc::GrpcStreamingResponse<super::route_guide::Feature>;

    fn RecordRoute(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::GrpcStreamingRequest<super::route_guide::Point>) -> ::grpc::GrpcSingleResponse<super::route_guide::RouteSummary>;

    fn RouteChat(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::GrpcStreamingRequest<super::route_guide::RouteNote>) -> ::grpc::GrpcStreamingResponse<super::route_guide::RouteNote>;
}

// client

pub struct RouteGuideClient {
    grpc_client: ::grpc::client::GrpcClient,
    method_GetFeature: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::route_guide::Point, super::route_guide::Feature>>,
    method_ListFeatures: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::route_guide::Rectangle, super::route_guide::Feature>>,
    method_RecordRoute: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::route_guide::Point, super::route_guide::RouteSummary>>,
    method_RouteChat: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::route_guide::RouteNote, super::route_guide::RouteNote>>,
}

impl RouteGuideClient {
    pub fn with_client(grpc_client: ::grpc::client::GrpcClient) -> Self {
        RouteGuideClient {
            grpc_client: grpc_client,
            method_GetFeature: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                name: "/proto.RouteGuide/GetFeature".to_string(),
                streaming: ::grpc::method::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
            }),
            method_ListFeatures: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                name: "/proto.RouteGuide/ListFeatures".to_string(),
                streaming: ::grpc::method::GrpcStreaming::ServerStreaming,
                req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
            }),
            method_RecordRoute: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                name: "/proto.RouteGuide/RecordRoute".to_string(),
                streaming: ::grpc::method::GrpcStreaming::ClientStreaming,
                req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
            }),
            method_RouteChat: ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                name: "/proto.RouteGuide/RouteChat".to_string(),
                streaming: ::grpc::method::GrpcStreaming::Bidi,
                req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
            }),
        }
    }

    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        ::grpc::client::GrpcClient::new(host, port, tls, conf).map(|c| {
            RouteGuideClient::with_client(c)
        })
    }
}

impl RouteGuide for RouteGuideClient {
    fn GetFeature(&self, o: ::grpc::GrpcRequestOptions, p: super::route_guide::Point) -> ::grpc::GrpcSingleResponse<super::route_guide::Feature> {
        self.grpc_client.call_unary(o, p, self.method_GetFeature.clone())
    }

    fn ListFeatures(&self, o: ::grpc::GrpcRequestOptions, p: super::route_guide::Rectangle) -> ::grpc::GrpcStreamingResponse<super::route_guide::Feature> {
        self.grpc_client.call_server_streaming(o, p, self.method_ListFeatures.clone())
    }

    fn RecordRoute(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::GrpcStreamingRequest<super::route_guide::Point>) -> ::grpc::GrpcSingleResponse<super::route_guide::RouteSummary> {
        self.grpc_client.call_client_streaming(o, p, self.method_RecordRoute.clone())
    }

    fn RouteChat(&self, o: ::grpc::GrpcRequestOptions, p: ::grpc::GrpcStreamingRequest<super::route_guide::RouteNote>) -> ::grpc::GrpcStreamingResponse<super::route_guide::RouteNote> {
        self.grpc_client.call_bidi(o, p, self.method_RouteChat.clone())
    }
}

// server

pub struct RouteGuideServer {
    pub grpc_server: ::grpc::server::GrpcServer,
}

impl ::std::ops::Deref for RouteGuideServer {
    type Target = ::grpc::server::GrpcServer;

    fn deref(&self) -> &Self::Target {
        &self.grpc_server
    }
}

impl RouteGuideServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : RouteGuide + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let service_definition = RouteGuideServer::new_service_def(h);
        RouteGuideServer {
            grpc_server: ::grpc::server::GrpcServer::new_plain(addr, conf, service_definition),
        }
    }

    pub fn new_pool<A : ::std::net::ToSocketAddrs, H : RouteGuide + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H, cpu_pool: ::futures_cpupool::CpuPool) -> Self {
        let service_definition = RouteGuideServer::new_service_def(h);
        RouteGuideServer {
            grpc_server: ::grpc::server::GrpcServer::new_plain_pool(addr, conf, service_definition, cpu_pool),
        }
    }

    pub fn new_service_def<H : RouteGuide + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::server::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::server::ServerServiceDefinition::new(
            vec![
                ::grpc::server::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                        name: "/proto.RouteGuide/GetFeature".to_string(),
                        streaming: ::grpc::method::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerUnary::new(move |o, p| handler_copy.GetFeature(o, p))
                    },
                ),
                ::grpc::server::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                        name: "/proto.RouteGuide/ListFeatures".to_string(),
                        streaming: ::grpc::method::GrpcStreaming::ServerStreaming,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerServerStreaming::new(move |o, p| handler_copy.ListFeatures(o, p))
                    },
                ),
                ::grpc::server::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                        name: "/proto.RouteGuide/RecordRoute".to_string(),
                        streaming: ::grpc::method::GrpcStreaming::ClientStreaming,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerClientStreaming::new(move |o, p| handler_copy.RecordRoute(o, p))
                    },
                ),
                ::grpc::server::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::method::MethodDescriptor {
                        name: "/proto.RouteGuide/RouteChat".to_string(),
                        streaming: ::grpc::method::GrpcStreaming::Bidi,
                        req_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::grpc_protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::server::MethodHandlerBidi::new(move |o, p| handler_copy.RouteChat(o, p))
                    },
                ),
            ],
        )
    }
}
