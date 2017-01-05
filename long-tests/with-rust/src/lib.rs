extern crate protobuf;
extern crate futures;
extern crate futures_cpupool;
extern crate grpc;

pub mod long_tests_pb;
pub mod long_tests_pb_grpc;

pub const TEST_HOST: &'static str = "localhost:23432";
