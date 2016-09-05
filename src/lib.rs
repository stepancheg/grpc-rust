extern crate protobuf;
extern crate solicit;
#[macro_use]
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;

pub mod client;
pub mod server;
mod grpc;
pub mod method;
pub mod grpc_protobuf;
pub mod marshall;
pub mod futures_grpc;
pub mod result;
pub mod error;
pub mod iter;
pub mod rt;

mod http_client;
mod http_server;
pub mod futures_misc;
mod misc;
mod solicit_async;
mod solicit_misc;
mod tokio_oneshot;
