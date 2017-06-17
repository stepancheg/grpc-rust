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

use grpc::*;

struct LongTestsServerImpl {
}

impl LongTests for LongTestsServerImpl {
    fn echo(&self, _o: RequestOptions, mut p: EchoRequest)
        -> grpc::SingleResponse<EchoResponse>
    {
        let mut resp = EchoResponse::new();
        resp.set_payload(p.take_payload());
        grpc::SingleResponse::completed(resp)
    }

    fn char_count(&self, _o: grpc::RequestOptions, p: grpc::StreamingRequest<CharCountRequest>)
        -> grpc::SingleResponse<CharCountResponse>
    {
        let r = p.0
            .map(|c| c.part.len() as u64)
            .fold(0, |a, b| futures::finished::<_, grpc::Error>(a + b))
            .map(|s| {
                let mut r = CharCountResponse::new();
                r.char_count = s;
                r
            });
        grpc::SingleResponse::no_metadata(r)
    }

    fn random_strings(&self, _o: grpc::RequestOptions, p: RandomStringsRequest)
        -> grpc::StreamingResponse<RandomStringsResponse>
    {
        let iter = iter::repeat(())
            .map(|_| {
                let s = "aabb".to_owned();
                let mut resp = RandomStringsResponse::new();
                resp.set_s(s);
                resp
            })
            .take(p.count as usize);
        grpc::StreamingResponse::iter(iter)
    }
}

fn main() {
    env_logger::init().unwrap();

    let _server = LongTestsServer::new(
        long_tests::TEST_HOST, Default::default(), LongTestsServerImpl {})
            .expect("server");

    loop {
        thread::park();
    }
}
