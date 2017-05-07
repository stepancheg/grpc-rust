#![allow(dead_code)]

use httpbis::server::HttpServer;
use httpbis::server::ServerTlsOption;
use httpbis::http_common::*;
use httpbis::Headers;


pub struct HttpServerEcho {
    server: HttpServer,
    pub port: u16,
}

struct EchoService {
}

impl HttpService for EchoService {
    fn new_request(&self, _headers: Headers, req: HttpPartFutureStreamSend) -> HttpResponse {
        let headers = Headers::ok_200();
        HttpResponse::headers_and_stream(headers, req)
    }
}

impl HttpServerEcho {
    pub fn new() -> HttpServerEcho {
        let http_server = HttpServer::new("[::1]:0", ServerTlsOption::Plain, Default::default(), EchoService {});
        let port = http_server.local_addr().port();
        HttpServerEcho {
            server: http_server,
            port: port,
        }
    }
}
