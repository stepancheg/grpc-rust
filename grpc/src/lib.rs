#[macro_use]
extern crate log;
#[macro_use]
extern crate futures;
extern crate bytes;
extern crate futures_cpupool;
extern crate tokio_core;
extern crate tls_api;
extern crate tls_api_stub;
extern crate tokio_tls_api;
extern crate base64;

// renamed to avoid name conflict with local protobuf library
extern crate protobuf as protobuf_lib;

extern crate httpbis;

mod futures_misc;
mod misc;

mod client;
mod server;
mod server_method;

mod assert_types;

mod grpc;
mod grpc_frame;
mod grpc_http_to_response;
mod result;
mod stream_item;
mod req;
mod resp;
mod chars;

mod method;
mod marshall;
mod futures_grpc;
mod error;
mod iter;
mod metadata;

pub mod rt;
pub mod protobuf;

pub mod for_test;


pub use error::Error;
pub use error::GrpcMessageError;
pub use result::Result;

pub use stream_item::ItemOrMetadata;

pub use client::Client;
pub use client::ClientConf;

pub use server::Server;
pub use server::ServerBuilder;
pub use server::ServerConf;
pub use server::GrpcHttpService;

pub use resp::SingleResponse;
pub use resp::StreamingResponse;

pub use req::RequestOptions;
pub use req::StreamingRequest;

pub use futures_grpc::GrpcStream;
pub use futures_grpc::GrpcFuture;

pub use metadata::Metadata;
pub use metadata::MetadataKey;
