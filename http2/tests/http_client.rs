use std::net;
use std::str;

extern crate bytes;
extern crate httpbis;
extern crate futures;
extern crate tokio_core;
extern crate tokio_tls;
#[macro_use]
extern crate log;
extern crate env_logger;

use futures::Future;

mod test_misc;

use test_misc::*;
use httpbis::for_test::*;

#[test]
fn stream_count_new() {
    env_logger::init().ok();

    let server = HttpServerTester::new();

    debug!("started server on {}", server.port());

    let client: HttpClient =
        HttpClient::new("::1", server.port(), false, Default::default()).expect("connect");

    let mut server_tester = server.accept();
    server_tester.recv_preface();
    server_tester.settings_xchg();

    let state: ConnectionStateSnapshot = client.dump_state().wait().expect("state");
    assert_eq!(0, state.streams.len());

    let req = client.start_post_simple_response("/foobar", (b"xxyy"[..]).to_owned());

    let headers = server_tester.recv_frame_headers_check(1, false);
    assert_eq!("POST", headers.get(":method"));
    assert_eq!("/foobar", headers.get(":path"));

    let data = server_tester.recv_frame_data_check(1, false);
    assert_eq!(b"xxyy", &data[..]);

    let headers = server_tester.recv_frame_data_check(1, true);
    assert!(headers.is_empty());

    let mut resp_headers = Headers::new();
    resp_headers.add(":status", "200");
    server_tester.send_headers(1, resp_headers, false);

    server_tester.send_data(1, b"aabb", true);

    let message = req.wait().expect("r");
    assert_eq!((b"aabb"[..]).to_owned(), message.body);

    let state: ConnectionStateSnapshot = client.dump_state().wait().expect("state");
    assert_eq!(0, state.streams.len(), "{:?}", state);
}


#[test]
fn stream_count() {
    env_logger::init().ok();

    let server = HttpServerEcho::new();

    debug!("started server on {}", server.port);

    let client: HttpClient =
        HttpClient::new("::1", server.port, false, Default::default()).expect("connect");

    let state: ConnectionStateSnapshot = client.dump_state().wait().expect("state");
    assert_eq!(0, state.streams.len());

    let message = client.start_post_simple_response("/foobar", (b"xxyy"[..]).to_owned())
        .wait()
        .expect("r");
    assert_eq!((b"xxyy"[..]).to_owned(), message.body);

    let state: ConnectionStateSnapshot = client.dump_state().wait().expect("state");
    assert_eq!(0, state.streams.len(), "{:?}", state);
}
