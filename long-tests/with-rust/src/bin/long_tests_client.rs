use std::thread;
use std::iter;

extern crate env_logger;

extern crate grpc;
extern crate long_tests;
extern crate futures;

use futures::stream::Stream;
use futures::stream;
use futures::Future;

use long_tests::long_tests_pb::*;
use long_tests::long_tests_pb_grpc::*;

use grpc::futures_grpc::GrpcStreamSend;
use grpc::futures_grpc::GrpcFutureSend;
use grpc::error::GrpcError;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("too few args")
    }

    let client = LongTestsClient::new("localhost", 23432, false).expect("init");

    let cmd = &args[1];
    if cmd == "echo" {

    } else {
        panic!("unknown command: {}", cmd);
    }
}
