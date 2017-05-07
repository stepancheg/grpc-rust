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

use httpbis::solicit::header::*;

use httpbis::for_test::*;

use test_misc::*;


#[test]
fn simple_new() {
    env_logger::init().ok();

    let server = HttpServerOneConn::new_fn(0, |_headers, req| {
        HttpResponse::headers_and_stream(Headers::ok_200(), req)
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
fn panic_in_handler() {
    env_logger::init().ok();

    let server = HttpServerOneConn::new_fn(0, |headers, _req| {
        if headers.path() == "/panic" {
            panic!("requested");
        } else {
            HttpResponse::headers_and_bytes(Headers::ok_200(), Bytes::from("hi there"))
        }
    });

    let mut tester = HttpConnectionTester::connect(server.port());
    tester.send_preface();
    tester.settings_xchg();

    {
        let resp = tester.get(1, "/hello");
        assert_eq!(200, resp.headers.status());
        assert_eq!(&b"hi there"[..], &resp.body[..]);
    }

    {
        let resp = tester.get(3, "/panic");
        assert_eq!(500, resp.headers.status());
    }

    {
        let resp = tester.get(5, "/world");
        assert_eq!(200, resp.headers.status());
        assert_eq!(&b"hi there"[..], &resp.body[..]);
    }

    assert_eq!(0, server.dump_state().streams.len());
}

#[test]
fn panic_in_stream() {
    env_logger::init().ok();

    let server = HttpServerOneConn::new_fn(0, |headers, _req| {
        if headers.path() == "/panic" {
            HttpResponse::from_stream(stream::iter((0..2).map(|i| {
                match i {
                    0 => Ok(HttpStreamPart::intermediate_headers(Headers::ok_200())),
                    _ => panic!("should reset stream"),
                }
            })))
        } else {
            HttpResponse::headers_and_bytes(Headers::ok_200(), Bytes::from("hi there"))
        }
    });

    let mut tester = HttpConnectionTester::connect(server.port());
    tester.send_preface();
    tester.settings_xchg();

    {
        let resp = tester.get(1, "/hello");
        assert_eq!(200, resp.headers.status());
        assert_eq!(&b"hi there"[..], &resp.body[..]);
    }

    {
        tester.send_get(3, "/panic");
        tester.recv_frame_headers_check(3, false);
        tester.recv_rst_frame();
    }

    {
        let resp = tester.get(5, "/world");
        assert_eq!(200, resp.headers.status());
        assert_eq!(&b"hi there"[..], &resp.body[..]);
    }

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
        HttpResponse::headers_and_bytes(Headers::ok_200(), Bytes::from(large_resp_copy.clone()))
    });

    let client = HttpClient::new("::1", server.port(), false, Default::default()).expect("connect");
    let resp = client.start_post("/foobar", "localhost", Bytes::from(&b""[..])).collect().wait().expect("wait");
    assert_eq!(large_resp.len(), resp.body.len());
    assert_eq!((large_resp.len(), &large_resp[..]), (resp.body.len(), &resp.body[..]));

    assert_eq!(0, server.dump_state().streams.len());
}
