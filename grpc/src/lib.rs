#[macro_use]
extern crate log;
#[macro_use]
extern crate futures;
extern crate base64;
extern crate bytes;
extern crate futures_cpupool;
extern crate tls_api;
extern crate tls_api_stub;
extern crate tokio_core;
extern crate tokio_tls_api;

extern crate httpbis;

mod futures_misc;
mod misc;

mod client;
mod client_stub;
mod common;
mod server;

mod proto;

mod assert_types;

mod chars;
mod or_static;
mod req;
mod resp;
mod result;
mod stream_item;

mod error;
mod futures_grpc;
mod iter;
pub mod marshall;
mod method;

pub mod prelude;

pub mod rt;

pub mod for_test;

pub use error::Error;
pub use error::GrpcMessageError;
pub use result::Result;

pub use stream_item::ItemOrMetadata;

pub use client::req_sink::ClientRequestSink;
pub use client::Client;
pub use client::ClientBuilder;
pub use client::ClientConf;

pub use client_stub::ClientStub;
pub use client_stub::ClientStubExt;

pub use server::ctx::ServerHandlerContext;
pub use server::req_handler::ServerRequest;
pub use server::req_single::ServerRequestSingle;
pub use server::req_stream::ServerRequestStream;
pub use server::resp_sink::ServerResponseSink;
pub use server::resp_unary_sink::ServerResponseUnarySink;
pub use server::Server;
pub use server::ServerBuilder;
pub use server::ServerConf;

pub use resp::SingleResponse;
pub use resp::StreamingResponse;

pub use req::RequestOptions;
pub use req::StreamingRequest;

pub use futures_grpc::GrpcFuture;
pub use futures_grpc::GrpcStream;

pub use proto::grpc_status::GrpcStatus;
pub use proto::metadata::Metadata;
pub use proto::metadata::MetadataKey;
