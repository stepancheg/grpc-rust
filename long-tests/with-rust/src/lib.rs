extern crate futures;

extern crate grpc;
extern crate grpc_protobuf;
extern crate protobuf;
extern crate tls_api;

pub mod long_tests_pb;
pub mod long_tests_pb_grpc;

pub const TEST_HOST: &'static str = "localhost:23432";
