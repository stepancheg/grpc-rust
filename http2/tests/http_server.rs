extern crate futures;
extern crate tokio_core;
extern crate tokio_tls;
extern crate httpbis;
#[macro_use]
extern crate log;

mod test_misc;

use std::sync::mpsc;
use std::net::ToSocketAddrs;
use std::thread;

use futures::Future;
use futures::stream;
use futures::stream::Stream;
use tokio_core::reactor;

use httpbis::solicit::Header;
use httpbis::solicit::HttpError;

use httpbis::for_test::*;

use test_misc::*;


#[test]
fn test() {
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

    let client: Http2Client = Http2Client::new("::1", server.port(), false).expect("connect");
    client.start_post_simple_response("/foobar", (&b"abcd"[..]).to_owned()).wait();

    assert_eq!(0, server.dump_state().streams.len());
}
