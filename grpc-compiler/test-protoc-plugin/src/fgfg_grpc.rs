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

pub trait Baz {
    fn qux(&self, o: ::grpc::RequestOptions, p: super::fgfg::FooBar) -> ::grpc::SingleResponse<super::fgfg::FooBar>;
}

// client

pub struct BazClient {
    grpc_client: ::grpc::Client,
    method_qux: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::fgfg::FooBar, super::fgfg::FooBar>>,
}

impl BazClient {
    pub fn with_client(grpc_client: ::grpc::Client) -> Self {
        BazClient {
            grpc_client: grpc_client,
            method_qux: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/Baz/qux".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }

    pub fn new_plain(host: &str, port: u16, conf: ::grpc::ClientConf) -> ::grpc::Result<Self> {
        ::grpc::Client::new_plain(host, port, conf).map(|c| {
            BazClient::with_client(c)
        })
    }
    pub fn new_tls<C : ::tls_api::TlsConnector>(host: &str, port: u16, conf: ::grpc::ClientConf) -> ::grpc::Result<Self> {
        ::grpc::Client::new_tls::<C>(host, port, conf).map(|c| {
            BazClient::with_client(c)
        })
    }
}

impl Baz for BazClient {
    fn qux(&self, o: ::grpc::RequestOptions, p: super::fgfg::FooBar) -> ::grpc::SingleResponse<super::fgfg::FooBar> {
        self.grpc_client.call_unary(o, p, self.method_qux.clone())
    }
}

// server

pub struct BazServer {
    pub grpc_server: ::grpc::Server,
}

impl ::std::ops::Deref for BazServer {
    type Target = ::grpc::Server;

    fn deref(&self) -> &Self::Target {
        &self.grpc_server
    }
}

impl BazServer {
    pub fn new<A : ::std::net::ToSocketAddrs, H : Baz + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::ServerConf, h: H) -> ::grpc::Result<Self> {
        let service_definition = BazServer::new_service_def(h);
        Ok(BazServer {
            grpc_server: ::grpc::Server::new_plain(addr, conf, service_definition)?,
        })
    }

    pub fn new_pool<A : ::std::net::ToSocketAddrs, H : Baz + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::ServerConf, h: H, cpu_pool: ::futures_cpupool::CpuPool) -> ::grpc::Result<Self> {
        let service_definition = BazServer::new_service_def(h);
        Ok(BazServer {
            grpc_server: ::grpc::Server::new_plain_pool(addr, conf, service_definition, cpu_pool)?,
        })
    }

    pub fn new_service_def<H : Baz + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/Baz",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/Baz/qux".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.qux(o, p))
                    },
                ),
            ],
        )
    }
}
