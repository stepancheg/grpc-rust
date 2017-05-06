use std::thread;
use std::iter;

extern crate env_logger;

extern crate grpc;
extern crate long_tests;
extern crate futures;

use futures::stream::Stream;
use futures::Future;

use long_tests::long_tests_pb::*;
use long_tests::long_tests_pb_grpc::*;

use grpc::futures_grpc::*;
use grpc::error::GrpcError;
use grpc::*;

struct LongTestsServerImpl {
}

impl LongTestsAsync for LongTestsServerImpl {
    fn echo(&self, mut p: EchoRequest)
        -> GrpcSingleResponse<EchoResponse>
    {
        let mut resp = EchoResponse::new();
        resp.set_payload(p.take_payload());
        GrpcSingleResponse::completed(resp)
    }

    fn char_count(&self, p: GrpcStreamSend<CharCountRequest>)
        -> GrpcSingleResponse<CharCountResponse>
    {
        let r = p
            .map(|c| c.part.len() as u64)
            .fold(0, |a, b| futures::finished::<_, GrpcError>(a + b))
            .map(|s| {
                let mut r = CharCountResponse::new();
                r.char_count = s;
                r
            });
        GrpcSingleResponse::no_metadata(r)
    }

    fn random_strings(&self, p: RandomStringsRequest)
        -> GrpcStreamingResponse<RandomStringsResponse>
    {
        let iter = iter::repeat(())
            .map(|_| {
                let s = "aabb".to_owned();
                let mut resp = RandomStringsResponse::new();
                resp.set_s(s);
                resp
            })
            .take(p.count as usize);
        GrpcStreamingResponse::iter(iter)
    }
}

fn main() {
    env_logger::init().unwrap();

    let _server = LongTestsAsyncServer::new(long_tests::TEST_HOST, Default::default(), LongTestsServerImpl {});

    loop {
        thread::park();
    }
}
