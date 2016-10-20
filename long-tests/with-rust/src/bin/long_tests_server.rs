use std::thread;

extern crate grpc;
extern crate long_tests;
extern crate futures;

use long_tests::long_tests_pb::*;
use long_tests::long_tests_pb_grpc::*;

struct LongTestsServerImpl {
}

impl LongTestsAsync for LongTestsServerImpl {
    fn echo(&self, mut p: EchoRequest) -> ::grpc::futures_grpc::GrpcFutureSend<EchoResponse> {
        let mut resp = EchoResponse::new();
        resp.set_payload(p.take_payload());
        Box::new(futures::finished(resp))
    }
}

fn main() {
    let _server = LongTestsAsyncServer::new(long_tests::TEST_PORT, LongTestsServerImpl {});

    loop {
        thread::park();
    }
}
