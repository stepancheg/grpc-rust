extern crate solicit;
extern crate futures;
extern crate tokio_core;
extern crate grpc;

use std::io;
use std::sync::mpsc;
use std::net::ToSocketAddrs;
use std::thread;

use futures::Future;
use futures::stream;
use futures::stream::Stream;
use tokio_core::reactor;
use tokio_core::net::*;

use solicit::http::HttpError;
use solicit::http::HttpResult;
use solicit::http::StaticHeader;

use grpc::*;
use grpc::for_test::*;


#[test]
fn test() {
    let (port_tx, port_rx) = mpsc::channel();

    thread::spawn(move || {
        let mut lp = reactor::Core::new().unwrap();

        let listener = TcpListener::bind(&("::1", 0).to_socket_addrs().unwrap().next().unwrap(), &lp.handle()).unwrap();

        port_tx.send(listener.local_addr().unwrap().port());

        let server_conn = lp.run(listener.incoming().into_future()).ok().expect("accept").0.unwrap().0;

        struct H {

        }

        impl HttpServerHandler for H {
            fn part(&mut self, part: HttpStreamPart) -> HttpResult<()> {
                println!("test: part: {:?}", part);
                Ok(())
            }

            fn end(&mut self) -> HttpResult<()> {
                println!("test: server got: end");
                Ok(())
            }
        }

        struct F {

        }

        impl HttpServerHandlerFactory for F {
            type RequestHandler = H;

            fn new_request(&mut self) -> (Self::RequestHandler, HttpStreamStreamSend) {
                (H {}, Box::new(stream::iter(vec![].into_iter())))
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
            client_complete_tx: mpsc::Sender<()>,
        }

        impl HttpClientResponseHandler for R {
            fn headers(&mut self, headers: Vec<StaticHeader>) -> bool {
                println!("test: client: response headers: {}", headers.len());
                true
            }

            fn data_frame(&mut self, chunk: Vec<u8>) -> bool {
                println!("test: client: response data frame: {}", chunk.len());
                true
            }

            fn trailers(&mut self, headers: Vec<StaticHeader>) -> bool {
                println!("test: client: response trailers: {}", headers.len());
                true
            }

            fn end(&mut self) {
                println!("test: client: end");
                self.client_complete_tx.send(());
            }
        }

        let resp = client.start_request(Vec::new(), stream_once_send((&b"abcd"[..]).to_owned()), R { client_complete_tx: client_complete_tx });

        client_lp.run(future).expect("client run");
    });

    client_complete_rx.recv().expect("client complete recv");
}
