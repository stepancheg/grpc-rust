extern crate solicit_fork as solicit;
extern crate futures;
extern crate tokio_core;
extern crate grpc;

use std::sync::mpsc;
use std::net::ToSocketAddrs;
use std::thread;

use futures::Future;
use futures::stream;
use futures::stream::Stream;
use tokio_core::reactor;
use tokio_core::net::*;

use solicit::http::StaticHeader;
use solicit::http::Header;
use solicit::http::HttpError;

use grpc::for_test::*;


#[test]
fn test() {
    let (port_tx, port_rx) = mpsc::channel();

    thread::spawn(move || {
        let mut lp = reactor::Core::new().unwrap();

        let listener = TcpListener::bind(&("::1", 0).to_socket_addrs().unwrap().next().unwrap(), &lp.handle()).unwrap();

        port_tx.send(listener.local_addr().unwrap().port()).unwrap();

        let server_conn = lp.run(listener.incoming().into_future()).ok().expect("accept").0.unwrap().0;

        struct F {
        }

        impl HttpService for F {
            fn new_request(&mut self, _headers: Vec<StaticHeader>, req: HttpStreamStreamSend) -> HttpStreamStreamSend {
                Box::new(future_flatten_to_stream(req
                    .fold(Vec::new(), |mut v, message| {
                        match message.content {
                            HttpStreamPartContent::Headers(..) => (),
                            HttpStreamPartContent::Data(d) => v.extend(d),
                        }

                        futures::finished::<_, HttpError>(v)
                    })
                    .and_then(|v| {
                        let mut r = Vec::new();
                        r.push(HttpStreamPart::intermediate_headers(
                            vec![
                                Header::new(":status", "200"),
                            ]
                        ));
                        r.push(HttpStreamPart::last_data(v));
                        Ok(stream::iter(r.into_iter().map(Ok)))
                    })))
            }
        }

        let http_server_future = HttpServerConnectionAsync::new(&lp.handle(), server_conn, F {});

        lp.run(http_server_future).expect("server run");
    });

    let port = port_rx.recv().expect("recv port");

    let (client_complete_tx, client_complete_rx) = mpsc::channel();

    thread::spawn(move || {
        let mut client_lp = reactor::Core::new().expect("core");

        let (client, future) = HttpClientConnectionAsync::new(client_lp.handle(), &("::1", port).to_socket_addrs().unwrap().next().unwrap());

        let resp = client.start_request(
            Vec::new(),
            stream_once_send((&b"abcd"[..]).to_owned()));

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
