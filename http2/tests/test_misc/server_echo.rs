#![allow(dead_code)]

use futures::stream;

use httpbis::server::Http2Server;
use httpbis::http_common::*;
use httpbis::Header;
use httpbis::StaticHeader;
use httpbis::futures_misc::*;


pub struct HttpServerEcho {
    server: Http2Server,
    pub port: u16,
}

struct EchoService {
}

impl HttpService for EchoService {
    fn new_request(&self, _headers: Vec<StaticHeader>, req: HttpStreamStreamSend) -> HttpStreamStreamSend {
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
        let http_server = Http2Server::new("::1:0", EchoService {});
        let port = http_server.local_addr().port();
        HttpServerEcho {
            server: http_server,
            port: port,
        }
    }
}
