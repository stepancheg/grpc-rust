#[macro_use]
extern crate log;
#[macro_use]
extern crate futures;
extern crate bytes;
extern crate futures_cpupool;
extern crate tokio_core;
extern crate tokio_tls;
extern crate base64;

extern crate protobuf;

extern crate httpbis;

pub mod client;
pub mod server;
mod grpc;
mod grpc_frame;
mod bytes_or_vec;
pub mod method;
pub mod grpc_protobuf;
pub mod marshall;
pub mod futures_grpc;
pub mod result;
pub mod error;
pub mod iter;
pub mod rt;
pub mod metadata;
mod chars;

pub mod for_test {
    pub use httpbis::server_conn::*;
    pub use httpbis::client_conn::*;
    pub use httpbis::http_common::*;
    pub use httpbis::solicit_async::*;
    pub use httpbis::futures_misc::*;
}
