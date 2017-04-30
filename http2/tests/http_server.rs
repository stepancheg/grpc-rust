extern crate bytes;
extern crate futures;
extern crate tokio_core;
extern crate tokio_tls;
extern crate httpbis;
#[macro_use]
extern crate log;
extern crate env_logger;

mod test_misc;

use std::io::Write as _Write;

use futures::Future;
use futures::stream;
use futures::stream::Stream;

use httpbis::solicit::Header;

use httpbis::for_test::*;

use test_misc::*;


#[test]
fn simple() {
    env_logger::init().ok();

    let server = HttpServerOneConn::new_fn(0, |_headers, req| {
        Box::new(req
            .collect()
            .and_then(|v| {
                let mut r = Vec::new();
                r.push(HttpStreamPart::intermediate_headers(
                    vec![
                        Header::new(":status", "200"),
                    ]
                ));
                r.push(HttpStreamPart::last_data(SimpleHttpMessage::from_parts(v).body));
                Ok(stream::iter(r.into_iter().map(Ok)))
            })
            .flatten_stream())
    });

    let client = HttpClient::new("::1", server.port(), false, Default::default()).expect("connect");
    client.start_post_simple_response("/foobar", (&b"abcd"[..]).to_owned()).wait().expect("wait");

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
        r.push(HttpStreamPart::intermediate_headers(
            vec![
                Header::new(":status", "200"),
            ]
        ));
        r.push(HttpStreamPart::intermediate_data(large_resp_copy.clone()));
        Box::new(stream::iter(r.into_iter().map(Ok)))
    });

    let client = HttpClient::new("::1", server.port(), false, Default::default()).expect("connect");
    let resp = client.start_post_simple_response("/foobar", (&b""[..]).to_owned()).wait().expect("wait");
    assert_eq!(large_resp.len(), resp.body.len());
    assert_eq!((large_resp.len(), &large_resp[..]), (resp.body.len(), &resp.body[..]));

    assert_eq!(0, server.dump_state().streams.len());
}
