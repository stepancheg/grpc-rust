extern crate futures;
extern crate grpc;
extern crate grpc_protobuf;
extern crate protobuf;

pub mod route_guide;
pub mod route_guide_grpc;
pub mod client;
pub mod server;
mod test;

pub const DEFAULT_PORT: u16 = 10000;
