use std::thread;

extern crate env_logger;

extern crate futures;
extern crate grpc;
extern crate long_tests;

use long_tests::long_tests_pb::*;
use long_tests::long_tests_pb_grpc::*;

use futures::Async;
use grpc::*;

struct LongTestsServerImpl {}

impl LongTests for LongTestsServerImpl {
    fn echo(
        &self,
        _: ServerHandlerContext,
        req: ServerRequestSingle<EchoRequest>,
        resp: ServerResponseUnarySink<EchoResponse>,
    ) -> grpc::Result<()> {
        let mut m = EchoResponse::new();
        m.set_payload(req.message.payload);
        resp.finish(m)
    }

    fn char_count(
        &self,
        _: ServerHandlerContext,
        req: ServerRequest<CharCountRequest>,
        resp: ServerResponseUnarySink<CharCountResponse>,
    ) -> grpc::Result<()> {
        let mut resp = Some(resp);
        let mut char_count = 0;
        req.register_stream_handler_basic(move |message| {
            Ok(match message {
                Some(m) => {
                    char_count += m.part.len();
                }
                None => {
                    resp.take().unwrap().finish({
                        let mut r = CharCountResponse::new();
                        r.char_count = char_count as u64;
                        r
                    })?;
                }
            })
        });
        Ok(())
    }

    fn random_strings(
        &self,
        c: ServerHandlerContext,
        req: ServerRequestSingle<RandomStringsRequest>,
        mut resp: ServerResponseSink<RandomStringsResponse>,
    ) -> grpc::Result<()> {
        let mut rem = req.message.count;
        c.spawn_poll_fn(move || loop {
            if let Async::NotReady = resp.poll()? {
                return Ok(Async::NotReady);
            }
            if rem == 0 {
                resp.send_trailers(Metadata::new())?;
                return Ok(Async::Ready(()));
            }
            let s = "aabb".to_owned();
            let mut m = RandomStringsResponse::new();
            m.set_s(s);
            resp.send_data(m)?;
            rem -= 1;
        });
        Ok(())
    }
}

fn main() {
    env_logger::init();

    let mut server = ServerBuilder::new_plain();
    server
        .http
        .set_addr(long_tests::TEST_HOST)
        .expect("set_addr");
    server.add_service(LongTestsServer::new_service_def(LongTestsServerImpl {}));
    let _server = server.build().expect("server");

    loop {
        thread::park();
    }
}
