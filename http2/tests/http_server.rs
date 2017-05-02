extern crate bytes;
extern crate futures;
extern crate native_tls;
extern crate tokio_core;
extern crate tokio_tls;
extern crate httpbis;
extern crate log;
extern crate env_logger;

use bytes::Bytes;

mod test_misc;

use std::io::Write as _Write;

use futures::Future;
use futures::stream;
use futures::stream::Stream;

use httpbis::solicit::header::*;

use httpbis::for_test::*;

use test_misc::*;


#[test]
fn simple_new() {
    env_logger::init().ok();

    let server = HttpServerOneConn::new_fn(0, |_headers, req| {
        Box::new(req
            .collect()
            .and_then(|v| {
                let mut r = Vec::new();
                r.push(HttpStreamPart::intermediate_headers(Headers::ok_200()));
                r.push(HttpStreamPart::last_data(SimpleHttpMessage::from_parts(v).body));
                Ok(stream::iter(r.into_iter().map(Ok)))
            })
            .flatten_stream())
    });

    let mut tester = HttpConnectionTester::connect(server.port());
    tester.send_preface();
    tester.settings_xchg();

    let mut headers = Headers::new();
    headers.add(":method", "GET");
    headers.add(":path", "/aabb");
    tester.send_headers(1, headers, false);

    tester.send_data(1, b"abcd", true);

    let recv_headers = tester.recv_frame_headers_check(1, false);
    assert_eq!("200", recv_headers.get(":status"));

    assert_eq!(&b"abcd"[..], &tester.recv_frame_data_check(1, true)[..]);

    assert_eq!(0, server.dump_state().streams.len());
}

#[test]
fn response_large() {
    env_logger::init().ok();

    let mut large_resp = Vec::new();
    while large_resp.len() < 100_000 {
        if large_resp.len() != 0 {
            write!(&mut large_resp, ",").unwrap();
        }
        let len = large_resp.len();
        write!(&mut large_resp, "{}", len).unwrap();
    }

    let large_resp_copy = large_resp.clone();

    let server = HttpServerOneConn::new_fn(0, move |_headers, _req| {
        let mut r = Vec::new();
        r.push(HttpStreamPart::intermediate_headers(Headers::ok_200()));
        r.push(HttpStreamPart::intermediate_data(Bytes::from(large_resp_copy.clone())));
        Box::new(stream::iter(r.into_iter().map(Ok)))
    });

    let client = HttpClient::new("::1", server.port(), false, Default::default()).expect("connect");
    let resp = client.start_post_simple_response("/foobar", "localhost", Bytes::from(&b""[..])).wait().expect("wait");
    assert_eq!(large_resp.len(), resp.body.len());
    assert_eq!((large_resp.len(), &large_resp[..]), (resp.body.len(), &resp.body[..]));

    assert_eq!(0, server.dump_state().streams.len());
}
