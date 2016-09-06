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
use solicit::http::session::Session;
use solicit::http::connection::HttpConnection;
use solicit::http::connection::SendStatus;
use solicit::http::connection::EndStream;
use solicit::http::connection::DataChunk;
use solicit::http::connection::SendFrame;
use solicit::http::priority::SimplePrioritizer;
use solicit::http::frame::RawFrame;
use solicit::http::frame::HttpSetting;
use solicit::http::frame::PingFrame;
use solicit::http::StreamId;
use solicit::http::HttpScheme;
use solicit::http::HttpError;
use solicit::http::ErrorCode;
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


trait HttpServerHandler: Send + 'static {
    fn headers(&mut self) -> HttpFuture<()>;
    fn data_chunk(&mut self) -> HttpFuture<()>;
    fn trailers(&mut self) -> HttpFuture<()>;
    fn end(&mut self) -> HttpFuture<()>;
}

trait HttpServerHandlerFactory: Send + 'static {
    type RequestHandler : HttpServerHandler;

    fn new_request(&mut self) -> (Self::RequestHandler, HttpFuture<ResponseHeaders>);
}

struct GrpcHttpServerStream<F : HttpServerHandlerFactory> {
    state: StreamState,
    request_handler: Option<F::RequestHandler>,
}

struct GrpcHttpServerSessionState<F : HttpServerHandlerFactory> {
    streams: HashMap<StreamId, GrpcHttpServerStream<F>>,
    next_stream_id: StreamId,
}


enum LastChunk {
    Empty,
    Headers(Vec<StaticHeader>),
    Chunk(Vec<u8>),
    Trailers(Vec<StaticHeader>),
}

struct MyServerSession<'a, F, S>
    where
        F : HttpServerHandlerFactory,
        S : SendFrame + 'a,
{
    state: &'a mut GrpcHttpServerSessionState<F>,
    sender: &'a mut S,
    last: Option<(StreamId, LastChunk)>,
}

impl<'a, F, S> MyServerSession<'a, F, S>
    where
        F : HttpServerHandlerFactory,
        S : SendFrame + 'a,
{
    fn new(state: &'a mut GrpcHttpServerSessionState<F>, sender: &'a mut S) -> MyServerSession<'a, F, S> {
        MyServerSession {
            state: state,
            sender: sender,
            last: None,
        }
    }
}

impl<'a, F, S> Session for MyServerSession<'a, F, S>
    where
        F : HttpServerHandlerFactory,
        S : SendFrame + 'a,
{
    fn new_data_chunk(&mut self, stream_id: StreamId, data: &[u8], conn: &mut HttpConnection) -> HttpResult<()> {
        unimplemented!()
    }

    fn new_headers<'n, 'v>(&mut self, stream_id: StreamId, headers: Vec<Header<'n, 'v>>, conn: &mut HttpConnection) -> HttpResult<()> {
        unimplemented!()
    }

    fn end_of_stream(&mut self, stream_id: StreamId, conn: &mut HttpConnection) -> HttpResult<()> {
        unimplemented!()
    }

    fn rst_stream(&mut self, stream_id: StreamId, error_code: ErrorCode, conn: &mut HttpConnection) -> HttpResult<()> {
        unimplemented!()
    }

    fn new_settings(&mut self, settings: Vec<HttpSetting>, conn: &mut HttpConnection) -> HttpResult<()> {
        unimplemented!()
    }

    fn on_ping(&mut self, ping: &PingFrame, conn: &mut HttpConnection) -> HttpResult<()> {
        unimplemented!()
    }

    fn on_pong(&mut self, ping: &PingFrame, conn: &mut HttpConnection) -> HttpResult<()> {
        unimplemented!()
    }
}




struct ServerInner<F : HttpServerHandlerFactory> {
    factory: F,
    conn: HttpConnection,
    session_state: GrpcHttpServerSessionState<F>,
    to_write_tx: tokio_core::Sender<ServerToWriteMessage<F>>,
}

struct ServerReadLoop<F>
    where F : HttpServerHandlerFactory
{
    read: TaskIoRead<TcpStream>,
    inner: TaskRcMut<ServerInner<F>>,
}


struct ServerReadToWriteMessage {
    buf: Vec<u8>,
}

enum ServerToWriteMessage<F : HttpServerHandlerFactory> {
    _Dummy(F),
    FromRead(ServerReadToWriteMessage),
}


impl<F : HttpServerHandlerFactory> ServerReadLoop<F> {
    fn recv_raw_frame(self) -> HttpFuture<(Self, RawFrame<'static>)> {
        let ServerReadLoop { read, inner } = self;
        recv_raw_frame(read)
            .map(|(read, frame)| (ServerReadLoop { read: read, inner: inner}, frame))
            .map_err(HttpError::from)
            .boxed()
    }

    fn read_process_frame(self) -> HttpFuture<Self> {
        self.recv_raw_frame()
            .and_then(move |(rl, frame)| rl.process_raw_frame(frame))
            .boxed()
    }

    fn process_raw_frame(self, raw_frame: RawFrame) -> HttpFuture<Self> {
        let last = self.inner.with(move |inner: &mut ServerInner<F>| {
            let mut send = VecSendFrame(Vec::new());
            let last = {
                let mut session = MyServerSession::new(&mut inner.session_state, &mut send);
                inner.conn.handle_next_frame(&mut OnceReceiveFrame::new(raw_frame), &mut session)
                    .unwrap();
                session.last.take()
            };

            if !send.0.is_empty() {
                inner.to_write_tx.send(ServerToWriteMessage::FromRead(ServerReadToWriteMessage { buf: send.0 }))
                    .expect("read to write");
            }

            last
        });

        if let Some((stream_id, last_chunk)) = last {
            //self.process_streams_after_handle_next_frame(stream_id, last_chunk)
            unimplemented!()
        } else {
            futures::finished(self).boxed()
        }
    }

    fn run(self) -> HttpFuture<()> {
        let stream = stream_repeat(());

        let future = stream.fold(self, |rl, _| {
            rl.read_process_frame()
        });

        future.map(|_| ()).boxed()
    }
}

struct ServerWriteLoop<F>
    where F : HttpServerHandlerFactory
{
    write: TaskIoWrite<TcpStream>,
    inner: TaskRcMut<ServerInner<F>>,
}

impl<F : HttpServerHandlerFactory> ServerWriteLoop<F> {
    fn run(self, requests: HttpStream<ServerToWriteMessage<F>>) -> HttpFuture<()> {
        futures::finished(()).boxed()
    }
}




struct HttpServerConnectionAsync<F : HttpServerHandlerFactory> {
    factory: F,
}

impl<F : HttpServerHandlerFactory> HttpServerConnectionAsync<F> {
    fn new(lh: LoopHandle, socket: TcpStream, factory : F) -> HttpFuture<()> {
        let (to_write_tx, to_write_rx) = lh.clone().channel();

        let to_write_rx = future_flatten_to_stream(to_write_rx).map_err(HttpError::from).boxed();

        let handshake = server_handshake(socket);

        let run = handshake.and_then(move |socket| {
            let (read, write) = TaskIo::new(socket).split();

            let inner = TaskRcMut::new(ServerInner {
                factory: factory,
                conn: HttpConnection::new(HttpScheme::Http),
                to_write_tx: to_write_tx.clone(),
                session_state: GrpcHttpServerSessionState {
                    next_stream_id: 2,
                    streams: HashMap::new(),
                },
            });

            let run_write = ServerWriteLoop { write: write, inner: inner.clone() }.run(to_write_rx);
            let run_read = ServerReadLoop { read: read, inner: inner.clone() }.run();

            run_write.join(run_read).map(|_| ())
        });

        run.boxed()
    }
}
