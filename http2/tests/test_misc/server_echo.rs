#![allow(dead_code)]

use futures::stream;
use futures::stream::Stream;

use httpbis::server::HttpServer;
use httpbis::server::ServerTlsOption;
use httpbis::http_common::*;
use httpbis::Header;
use httpbis::Headers;


pub struct HttpServerEcho {
    server: HttpServer,
    pub port: u16,
}

struct EchoService {
}

impl HttpService for EchoService {
    fn new_request(&self, _headers: Headers, req: HttpPartFutureStreamSend) -> HttpPartFutureStreamSend {
        let headers = stream::once(Ok(HttpStreamPart::intermediate_headers(Headers(vec![
            Header::new(":status", "200"),
        ]))));
        Box::new(headers.chain(req))
    }
}

impl HttpServerEcho {
    pub fn new() -> HttpServerEcho {
        let http_server = HttpServer::new("::1:0", ServerTlsOption::Plain, Default::default(), EchoService {});
        let port = http_server.local_addr().port();
        HttpServerEcho {
            server: http_server,
            port: port,
        }
    }
}
