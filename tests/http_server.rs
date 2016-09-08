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
use futures::stream::BoxStream;
use tokio_core::Loop;

use solicit::http::HttpError;
use solicit::http::HttpResult;
use solicit::http::StaticHeader;

use grpc::*;
use grpc::for_test::*;


#[test]
fn test() {
    let (port_tx, port_rx) = mpsc::channel();

    thread::spawn(move || {
        let mut server_lp = Loop::new().unwrap();

        let listener = server_lp.handle().tcp_listen(&("::1", 0).to_socket_addrs().unwrap().next().unwrap());
        let listener = server_lp.run(listener).unwrap();

        port_tx.send(listener.local_addr().unwrap().port());

        let server_conn = server_lp.run(listener.incoming().into_future()).ok().expect("accept").0.unwrap().0;

        struct H {

        }

        impl HttpServerHandler for H {
            fn headers(&mut self, headers: Vec<StaticHeader>) -> HttpResult<()> {
                println!("test: server got: headers: {}", headers.len());
                Ok(())
            }

            fn data_frame(&mut self, data: &[u8]) -> HttpResult<()> {
                println!("test: server got: data frame: {}", data.len());
                Ok(())
            }

            fn trailers(&mut self) -> HttpResult<()> {
                println!("test: server got: trailers");
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

            fn new_request(&mut self) -> (Self::RequestHandler, HttpFuture<ResponseHeaders>) {
                (H {}, futures::finished(ResponseHeaders {
                    headers: Vec::new(),
                    after: futures::failed(HttpError::IoError(io::Error::new(io::ErrorKind::Other, "aa"))).boxed(),
                }).boxed())
            }
        }

        let http_server_future = HttpServerConnectionAsync::new(server_lp.handle(), server_conn, F {});

        server_lp.run(http_server_future).expect("server run");
    });

    let port = port_rx.recv().expect("recv port");

    let mut client_lp = Loop::new().unwrap();

    let (client, future) = HttpClientConnectionAsync::new(client_lp.handle(), &("::1", port).to_socket_addrs().unwrap().next().unwrap());

    struct R {
    }

    impl HttpClientResponseHandler for R {
        fn headers(&mut self, headers: Vec<StaticHeader>) -> bool {
            println!("test: client: response headers");
            true
        }

        fn data_frame(&mut self, chunk: Vec<u8>) -> bool {
            unimplemented!()
        }

        fn trailers(&mut self, headers: Vec<StaticHeader>) -> bool {
            unimplemented!()
        }

        fn end(&mut self) {
            unimplemented!()
        }
    }

    let resp = client.start_request(Vec::new(), stream_once((&b"abcd"[..]).to_owned()), R {});

//    client_lp.run(future).expect("client run");
//
//    println!("test: client loop complete");
//
//    resp.wait().unwrap();
}
