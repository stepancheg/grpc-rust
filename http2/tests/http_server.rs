extern crate solicit_fork as solicit;
extern crate futures;
extern crate tokio_core;
extern crate tokio_tls;
extern crate http2;
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

use solicit::http::Header;
use solicit::http::HttpError;

use http2::for_test::*;

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

    let port = server.port();

    let (client_complete_tx, client_complete_rx) = mpsc::channel();

    thread::spawn(move || {
        let mut client_lp = reactor::Core::new().expect("core");

        let (client, future) = HttpClientConnectionAsync::new_plain(client_lp.handle(), &("::1", port).to_socket_addrs().unwrap().next().unwrap());

        let resp = client.start_request(
            Vec::new(),
            Box::new(stream::once(Ok((&b"abcd"[..]).to_owned()))));

        let request_future = resp.fold(Vec::new(), move |mut v, part| {
            match part.content {
                HttpStreamPartContent::Headers(..) => (),
                HttpStreamPartContent::Data(data) => v.extend(data),
            }
            if part.last {
                client_complete_tx.send(v.clone()).unwrap()
            }
            futures::finished::<_, HttpError>(v)
        }).map(|_| ());

        client_lp.run(future.select(request_future)).ok();
    });

    assert_eq!(&b"abcd"[..], &client_complete_rx.recv().expect("client complete recv")[..]);
}
