use std::thread;

use long_tests::long_tests_pb::*;
use long_tests::long_tests_pb_grpc::*;

use grpc::*;
use std::task::Poll;

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
        c.spawn_poll_fn(move |cx| loop {
            if let Poll::Pending = resp.poll(cx)? {
                return Poll::Pending;
            }
            if rem == 0 {
                resp.send_trailers(Metadata::new())?;
                return Poll::Ready(Ok(()));
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
