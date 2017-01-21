#![allow(dead_code)]

use futures::stream;

use httpbis::server::HttpServer;
use httpbis::http_common::*;
use httpbis::Header;
use httpbis::StaticHeader;
use httpbis::futures_misc::*;


pub struct HttpServerEcho {
    server: HttpServer,
    pub port: u16,
}

struct EchoService {
}

impl HttpService for EchoService {
    fn new_request(&self, _headers: Vec<StaticHeader>, req: HttpPartFutureStreamSend) -> HttpPartFutureStreamSend {
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
        let http_server = HttpServer::new("::1:0", Default::default(), EchoService {});
        let port = http_server.local_addr().port();
        HttpServerEcho {
            server: http_server,
            port: port,
        }
    }
}
