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

pub mod client;
pub mod server;

mod grpc;
mod grpc_frame;
mod grpc_http_to_response;
mod result;
mod stream_item;
mod req;
mod resp;
mod chars;

pub mod method;
pub mod marshall;
pub mod futures_grpc;
pub mod error;
pub mod iter;
pub mod rt;
pub mod metadata;

pub mod protobuf;


pub use error::Error;
pub use result::Result;

pub use stream_item::ItemOrMetadata;

pub use client::Client;
pub use client::ClientConf;

pub use server::Server;
pub use server::ServerConf;

pub use resp::SingleResponse;
pub use resp::StreamingResponse;

pub use req::RequestOptions;
pub use req::StreamingRequest;

pub use metadata::Metadata;
