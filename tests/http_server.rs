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

        impl HttpServerHandlerFactory for F {
            fn new_request(&mut self, req: HttpStreamStreamSend) -> HttpStreamStreamSend {
                future_flatten_to_stream(req
                    .fold(Vec::new(), |mut v, message| {
                        match message {
                            HttpStreamPart::Headers(..) => (),
                            HttpStreamPart::Data(d) => v.extend(d),
                        }

                        futures::finished::<_, HttpError>(v)
                    })
                    .and_then(|v| {
                        let mut r = Vec::new();
                        r.push(HttpStreamPart::Headers(
                            vec![
                                Header::new(":status", "200"),
                            ]
                        ));
                        r.push(HttpStreamPart::Data(v));
                        Ok(stream::iter(r.into_iter().map(Ok)))
                    }))
            }
        }

        let http_server_future = HttpServerConnectionAsync::new(lp.handle(), server_conn, F {});

        lp.run(http_server_future).expect("server run");
    });

    let port = port_rx.recv().expect("recv port");

    let (client_complete_tx, client_complete_rx) = mpsc::channel();

    thread::spawn(move || {
        let mut client_lp = reactor::Core::new().expect("core");

        let (client, future) = HttpClientConnectionAsync::new(client_lp.handle(), &("::1", port).to_socket_addrs().unwrap().next().unwrap());

        struct R {
            response: Vec<u8>,
            client_complete_tx: mpsc::Sender<Vec<u8>>,
        }

        impl HttpClientResponseHandler for R {
            fn headers(&mut self, headers: Vec<StaticHeader>) -> bool {
                println!("test: client: response headers: {}", headers.len());
                true
            }

            fn data_frame(&mut self, chunk: Vec<u8>) -> bool {
                println!("test: client: response data frame: {}", chunk.len());
                self.response.extend(&chunk);
                true
            }

            fn trailers(&mut self, headers: Vec<StaticHeader>) -> bool {
                println!("test: client: response trailers: {}", headers.len());
                true
            }

            fn end(&mut self) {
                println!("test: client: end");
                self.client_complete_tx.send(self.response.clone()).unwrap();
            }
        }

        let _resp = client.start_request(
            Vec::new(),
            stream_once_send((&b"abcd"[..]).to_owned()),
            R { client_complete_tx: client_complete_tx, response: Vec::new() });

        client_lp.run(future).expect("client run");
    });

    assert_eq!(&b"abcd"[..], &client_complete_rx.recv().expect("client complete recv")[..]);
}
