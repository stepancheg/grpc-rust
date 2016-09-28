extern crate protobuf;
//extern crate solicit;
extern crate solicit_fork as solicit;
extern crate hpack;
#[macro_use]
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;
extern crate tokio_tls;

pub mod client;
pub mod server;
mod grpc;
mod grpc_frame;
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
mod http_common;
pub mod futures_misc;
mod misc;
mod solicit_async;
mod solicit_misc;
mod tokio_oneshot;
mod assert_types;

pub mod for_test {
    pub use http_server::*;
    pub use http_client::*;
    pub use http_common::*;
    pub use solicit_async::*;
    pub use futures_misc::*;
}
