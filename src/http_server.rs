use std::marker;
use std::cmp;
use std::mem;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::iter::FromIterator;

use solicit::http::client::RequestStream;
use solicit::http::client::ClientSession;
use solicit::http::session::Client;
use solicit::http::session::SessionState;
use solicit::http::session::DefaultSessionState;
use solicit::http::session::DefaultStream;
use solicit::http::session::StreamState;
use solicit::http::session::StreamDataError;
use solicit::http::session::StreamDataChunk;
use solicit::http::session::Stream as solicit_Stream;
use solicit::http::session::StreamIter;
use solicit::http::connection::HttpConnection;
use solicit::http::connection::SendStatus;
use solicit::http::connection::EndStream;
use solicit::http::connection::DataChunk;
use solicit::http::priority::SimplePrioritizer;
use solicit::http::frame::RawFrame;
use solicit::http::StreamId;
use solicit::http::HttpScheme;
use solicit::http::HttpError;
use solicit::http::HttpResult;
use solicit::http::Header;
use solicit::http::StaticHeader;

use futures;
use futures::Future;
use futures::stream::Stream;

use tokio_core;
use tokio_core::TcpStream;
use tokio_core::io as tokio_io;
use tokio_core::io::TaskIo;
use tokio_core::io::TaskIoRead;
use tokio_core::io::TaskIoWrite;
use tokio_core::LoopHandle;

use error::*;
use futures_misc::*;

use solicit_async::*;
use solicit_misc::*;


enum AfterHeaders {
    DataChunk(Vec<u8>, HttpFuture<AfterHeaders>),
    Trailers(Vec<StaticHeader>),
}

struct ResponseHeaders {
    headers: Vec<StaticHeader>,
    after: HttpFuture<AfterHeaders>,
}


trait HttpServerRequestHandler {
    fn headers(&mut self) -> HttpFuture<()>;
    fn data_chunk(&mut self) -> HttpFuture<()>;
    fn trailers(&mut self) -> HttpFuture<()>;
    fn end(&mut self) -> HttpFuture<()>;
}

trait HttpServerRequestHandlerFactory {
    type RequestHandler : HttpServerRequestHandler;

    fn new_request(&mut self) -> (Self::RequestHandler, HttpFuture<ResponseHeaders>);
}


struct HttpServerConnectionAsync<F : HttpServerRequestHandlerFactory> {
    factory: F,
}

impl<F : HttpServerRequestHandlerFactory> HttpServerConnectionAsync<F> {
    fn new(factory : F) -> Self {
        panic!();
    }
}
