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


fn single_num_arg_or(cmd_args: &[String], or: u64) -> u64 {
    if cmd_args.len() == 0 {
        or
    } else if cmd_args.len() == 1 {
        cmd_args[0].parse().expect("failed to parse as u64")
    } else {
        panic!("too many args");
    }
}


fn run_echo(client: AsyncLongTestsClient, cmd_args: &[String]) {
    let count = single_num_arg_or(cmd_args, 1);

    println!("running {} iterations of echo", count);

    for i in 0..count {
        let payload = format!("payload {}", i);

        let mut req = EchoRequest::new();
        req.set_payload(payload.clone());

        let r = client.echo(req).wait().expect("failed to get echo response");

        assert!(payload == r.get_payload());
    }

    println!("done");
}


fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("too few args")
    }

    let client = AsyncLongTestsClient::new("localhost", 23432, false, Default::default()).expect("init");

    let cmd = &args[1];
    let cmd_args = &args[2..];
    if cmd == "echo" {
        run_echo(client, cmd_args);
    } else {
        panic!("unknown command: {}", cmd);
    }
}
