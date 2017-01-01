#![allow(dead_code)]

use std::net::ToSocketAddrs;
use std::sync::mpsc;
use std::thread;

use futures::stream;

use http2::server::Http2Server;
use http2::http_common::*;
use http2::Header;
use http2::StaticHeader;
use http2::futures_misc::*;


pub struct HttpServerEcho {
    server: Http2Server,
    pub port: u16,
}

struct EchoService {
}

impl HttpService for EchoService {
    fn new_request(&mut self, headers: Vec<StaticHeader>, req: HttpStreamStreamSend) -> HttpStreamStreamSend {
        Box::new(stream_concat(
            stream::once(Ok(HttpStreamPart::intermediate_headers(vec![
                Header::new(":status", "200"),
            ]))),
            req,
        ))
    }
}

impl HttpServerEcho {
    pub fn new() -> HttpServerEcho {
        let http_server = Http2Server::new(0, || EchoService {});
        let port = http_server.local_addr().port();
        HttpServerEcho {
            server: http_server,
            port: port,
        }
    }
}
