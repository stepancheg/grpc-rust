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
    fn get_feature(&self, p: super::route_guide::Point) -> ::grpc::result::GrpcResult<super::route_guide::Feature>;

    fn list_features(&self, p: super::route_guide::Rectangle) -> ::grpc::iter::GrpcIterator<super::route_guide::Feature>;

    fn record_route(&self, p: ::grpc::iter::GrpcIterator<super::route_guide::Point>) -> ::grpc::result::GrpcResult<super::route_guide::RouteSummary>;

    fn route_chat(&self, p: ::grpc::iter::GrpcIterator<super::route_guide::RouteNote>) -> ::grpc::iter::GrpcIterator<super::route_guide::RouteNote>;
}

pub trait RouteGuideAsync {
    fn get_feature(&self, p: super::route_guide::Point) -> ::grpc::futures_grpc::GrpcFutureSend<super::route_guide::Feature>;

    fn list_features(&self, p: super::route_guide::Rectangle) -> ::grpc::futures_grpc::GrpcStreamSend<super::route_guide::Feature>;

    fn record_route(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::route_guide::Point>) -> ::grpc::futures_grpc::GrpcFutureSend<super::route_guide::RouteSummary>;

    fn route_chat(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::route_guide::RouteNote>) -> ::grpc::futures_grpc::GrpcStreamSend<super::route_guide::RouteNote>;
}

// sync client

pub struct RouteGuideClient {
    async_client: RouteGuideAsyncClient,
}

impl RouteGuideClient {
    pub fn new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self> {
        RouteGuideAsyncClient::new(host, port, tls, conf).map(|c| {
            RouteGuideClient {
                async_client: c,
            }
        })
    }
}

impl RouteGuide for RouteGuideClient {
    fn get_feature(&self, p: super::route_guide::Point) -> ::grpc::result::GrpcResult<super::route_guide::Feature> {
        ::futures::Future::wait(self.async_client.get_feature(p))
    }

    fn list_features(&self, p: super::route_guide::Rectangle) -> ::grpc::iter::GrpcIterator<super::route_guide::Feature> {
        ::grpc::rt::stream_to_iter(self.async_client.list_features(p))
    }

    fn record_route(&self, p: ::grpc::iter::GrpcIterator<super::route_guide::Point>) -> ::grpc::result::GrpcResult<super::route_guide::RouteSummary> {
        let p = ::futures::stream::Stream::boxed(::futures::stream::iter(::std::iter::IntoIterator::into_iter(p)));
        ::futures::Future::wait(self.async_client.record_route(p))
    }

    fn route_chat(&self, p: ::grpc::iter::GrpcIterator<super::route_guide::RouteNote>) -> ::grpc::iter::GrpcIterator<super::route_guide::RouteNote> {
        let p = ::futures::stream::Stream::boxed(::futures::stream::iter(::std::iter::IntoIterator::into_iter(p)));
        ::grpc::rt::stream_to_iter(self.async_client.route_chat(p))
    }
}

// async client

pub struct RouteGuideAsyncClient {
    grpc_client: ::grpc::client::GrpcClient,
    method_GetFeature: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::route_guide::Point, super::route_guide::Feature>>,
    method_ListFeatures: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::route_guide::Rectangle, super::route_guide::Feature>>,
    method_RecordRoute: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::route_guide::Point, super::route_guide::RouteSummary>>,
    method_RouteChat: ::std::sync::Arc<::grpc::method::MethodDescriptor<super::route_guide::RouteNote, super::route_guide::RouteNote>>,
}

impl RouteGuideAsyncClient {
    pub fn with_client(grpc_client: ::grpc::client::GrpcClient) -> Self {
        RouteGuideAsyncClient {
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
            RouteGuideAsyncClient::with_client(c)
        })
    }
}

impl RouteGuideAsync for RouteGuideAsyncClient {
    fn get_feature(&self, p: super::route_guide::Point) -> ::grpc::futures_grpc::GrpcFutureSend<super::route_guide::Feature> {
        self.grpc_client.call_unary(p, self.method_GetFeature.clone())
    }

    fn list_features(&self, p: super::route_guide::Rectangle) -> ::grpc::futures_grpc::GrpcStreamSend<super::route_guide::Feature> {
        self.grpc_client.call_server_streaming(p, self.method_ListFeatures.clone())
    }

    fn record_route(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::route_guide::Point>) -> ::grpc::futures_grpc::GrpcFutureSend<super::route_guide::RouteSummary> {
        self.grpc_client.call_client_streaming(p, self.method_RecordRoute.clone())
    }

    fn route_chat(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::route_guide::RouteNote>) -> ::grpc::futures_grpc::GrpcStreamSend<super::route_guide::RouteNote> {
        self.grpc_client.call_bidi(p, self.method_RouteChat.clone())
    }
}

// sync server

pub struct RouteGuideServer {
    async_server: RouteGuideAsyncServer,
}

impl ::std::ops::Deref for RouteGuideServer {
    type Target = RouteGuideAsyncServer;

    fn deref(&self) -> &Self::Target {
        &self.async_server
    }
}

struct RouteGuideServerHandlerToAsync {
    handler: ::std::sync::Arc<RouteGuide + Send + Sync>,
    cpupool: ::futures_cpupool::CpuPool,
}

impl RouteGuideAsync for RouteGuideServerHandlerToAsync {
    fn get_feature(&self, p: super::route_guide::Point) -> ::grpc::futures_grpc::GrpcFutureSend<super::route_guide::Feature> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_unary(&self.cpupool, p, move |p| {
            h.get_feature(p)
        })
    }

    fn list_features(&self, p: super::route_guide::Rectangle) -> ::grpc::futures_grpc::GrpcStreamSend<super::route_guide::Feature> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_server_streaming(&self.cpupool, p, move |p| {
            h.list_features(p)
        })
    }

    fn record_route(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::route_guide::Point>) -> ::grpc::futures_grpc::GrpcFutureSend<super::route_guide::RouteSummary> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_client_streaming(&self.cpupool, p, move |p| {
            h.record_route(p)
        })
    }

    fn route_chat(&self, p: ::grpc::futures_grpc::GrpcStreamSend<super::route_guide::RouteNote>) -> ::grpc::futures_grpc::GrpcStreamSend<super::route_guide::RouteNote> {
        let h = self.handler.clone();
        ::grpc::rt::sync_to_async_bidi(&self.cpupool, p, move |p| {
            h.route_chat(p)
        })
    }
}

impl RouteGuideServer {
    pub fn new_plain<A : ::std::net::ToSocketAddrs, H : RouteGuide + Send + Sync + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let h = RouteGuideServerHandlerToAsync {
            cpupool: ::futures_cpupool::CpuPool::new_num_cpus(),
            handler: ::std::sync::Arc::new(h),
        };
        RouteGuideServer {
            async_server: RouteGuideAsyncServer::new(addr, conf, h),
        }
    }
}

// async server

pub struct RouteGuideAsyncServer {
    pub grpc_server: ::grpc::server::GrpcServer,
}

impl ::std::ops::Deref for RouteGuideAsyncServer {
    type Target = ::grpc::server::GrpcServer;

    fn deref(&self) -> &Self::Target {
        &self.grpc_server
    }
}

impl RouteGuideAsyncServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : RouteGuideAsync + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self {
        let service_definition = RouteGuideAsyncServer::new_service_def(h);
        RouteGuideAsyncServer {
            grpc_server: ::grpc::server::GrpcServer::new_plain(addr, conf, service_definition),
        }
    }

    pub fn new_service_def<H : RouteGuideAsync + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::server::ServerServiceDefinition {
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
                        ::grpc::server::MethodHandlerUnary::new(move |p| handler_copy.get_feature(p))
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
                        ::grpc::server::MethodHandlerServerStreaming::new(move |p| handler_copy.list_features(p))
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
                        ::grpc::server::MethodHandlerClientStreaming::new(move |p| handler_copy.record_route(p))
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
                        ::grpc::server::MethodHandlerBidi::new(move |p| handler_copy.route_chat(p))
                    },
                ),
            ],
        )
    }
}
