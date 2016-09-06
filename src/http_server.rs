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


pub enum AfterHeaders {
    DataChunk(Vec<u8>, HttpFuture<AfterHeaders>),
    Trailers(Vec<StaticHeader>),
}

pub struct ResponseHeaders {
    pub headers: Vec<StaticHeader>,
    pub after: HttpFuture<AfterHeaders>,
}


// TODO: async
pub trait HttpServerHandler: Send + 'static {
    fn headers(&mut self, headers: Vec<StaticHeader>) -> HttpResult<()>;
    fn data_frame(&mut self, data: &[u8]) -> HttpResult<()>;
    fn trailers(&mut self) -> HttpResult<()>;
    fn end(&mut self) -> HttpResult<()>;
}

pub trait HttpServerHandlerFactory: Send + 'static {
    type RequestHandler : HttpServerHandler;

    fn new_request(&mut self) -> (Self::RequestHandler, HttpFuture<ResponseHeaders>);
}

struct GrpcHttpServerStream<F : HttpServerHandlerFactory> {
    state: StreamState,
    request_handler: Option<F::RequestHandler>,
}

impl<F : HttpServerHandlerFactory> GrpcHttpServerStream<F> {
    fn on_rst_stream(&mut self, _error_code: ErrorCode) {
        self.close();
    }

    fn close(&mut self) {
        self.set_state(StreamState::Closed);
    }

    fn close_local(&mut self) {
        let next = match self.state() {
            StreamState::HalfClosedRemote => StreamState::Closed,
            _ => StreamState::HalfClosedLocal,
        };
        self.set_state(next);
    }

    fn close_remote(&mut self) {
        let next = match self.state() {
            StreamState::HalfClosedLocal => StreamState::Closed,
            _ => StreamState::HalfClosedRemote,
        };
        self.set_state(next);
    }

    fn is_closed(&self) -> bool {
        self.state() == StreamState::Closed
    }

    fn is_closed_local(&self) -> bool {
        match self.state() {
            StreamState::HalfClosedLocal | StreamState::Closed => true,
            _ => false,
        }
    }

    fn is_closed_remote(&self) -> bool {
        match self.state() {
            StreamState::HalfClosedRemote | StreamState::Closed => true,
            _ => false,
        }
    }

    fn new_data_chunk(&mut self, data: &[u8]) {
        self.request_handler.as_mut().unwrap().data_frame(data).expect("xxx");
    }

    fn set_headers<'n, 'v>(&mut self, headers: Vec<Header<'n, 'v>>) {
        let headers = headers.into_iter().map(|h| Header::new(h.name().to_owned(), h.value().to_owned())).collect();
        self.request_handler.as_mut().unwrap().headers(headers).expect("xxx");
    }

    fn set_state(&mut self, state: StreamState) {
        self.state = state;
    }

    fn state(&self) -> StreamState {
        self.state
    }

}

struct GrpcHttpServerSessionState<F : HttpServerHandlerFactory> {
    factory: F,
    streams: HashMap<StreamId, GrpcHttpServerStream<F>>,
}

impl<F : HttpServerHandlerFactory> GrpcHttpServerSessionState<F> {
    fn insert_stream(&mut self, stream_id: StreamId, stream: GrpcHttpServerStream<F>) -> &mut GrpcHttpServerStream<F> {
        self.streams.insert(stream_id, stream);
        self.streams.get_mut(&stream_id).unwrap()
    }

    fn get_stream_ref(&self, stream_id: StreamId) -> Option<&GrpcHttpServerStream<F>> {
        self.streams.get(&stream_id)
    }

    fn get_stream_mut(&mut self, stream_id: StreamId) -> Option<&mut GrpcHttpServerStream<F>> {
        self.streams.get_mut(&stream_id)
    }

    fn remove_stream(&mut self, stream_id: StreamId) -> Option<GrpcHttpServerStream<F>> {
        self.streams.remove(&stream_id)
    }

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
    fn new_data_chunk(&mut self,
                      stream_id: StreamId,
                      data: &[u8],
                      _: &mut HttpConnection)
                      -> HttpResult<()> {
        let mut stream = match self.state.get_stream_mut(stream_id) {
            None => {
                return Ok(());
            }
            Some(stream) => stream,
        };
        // Now let the stream handle the data chunk
        stream.new_data_chunk(data);
        Ok(())
    }

    fn new_headers<'n, 'v>(&mut self,
                           stream_id: StreamId,
                           headers: Vec<Header<'n, 'v>>,
                           _conn: &mut HttpConnection)
                           -> HttpResult<()> {
        if let Some(stream) = self.state.get_stream_mut(stream_id) {
            // This'd correspond to having received trailers...
            stream.set_headers(headers);
            return Ok(());
        };

        let (handler, response) = self.state.factory.new_request();
        // TODO: send response to writer

        // New stream initiated by the client
        let mut stream = GrpcHttpServerStream {
            state: StreamState::Open,
            request_handler: Some(handler),
        };
        let stream = self.state.insert_stream(stream_id, stream);
        stream.set_headers(headers);
        Ok(())
    }

    fn end_of_stream(&mut self, stream_id: StreamId, _: &mut HttpConnection) -> HttpResult<()> {
        let mut stream = match self.state.get_stream_mut(stream_id) {
            None => {
                return Ok(());
            }
            Some(stream) => stream,
        };
        stream.close_remote();
        Ok(())
    }

    fn rst_stream(&mut self,
                  stream_id: StreamId,
                  error_code: ErrorCode,
                  _: &mut HttpConnection)
                  -> HttpResult<()> {
        self.state.get_stream_mut(stream_id).map(|stream| stream.on_rst_stream(error_code));
        Ok(())
    }

    fn new_settings(&mut self,
                    _settings: Vec<HttpSetting>,
                    conn: &mut HttpConnection)
                    -> HttpResult<()> {
        conn.sender(self.sender).send_settings_ack()
    }

    fn on_ping(&mut self, ping: &PingFrame, conn: &mut HttpConnection) -> HttpResult<()> {
        conn.sender(self.sender).send_ping_ack(ping.opaque_data())
    }

    fn on_pong(&mut self, _ping: &PingFrame, _conn: &mut HttpConnection) -> HttpResult<()> {
        Ok(())
    }
}




struct ServerInner<F : HttpServerHandlerFactory> {
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




pub struct HttpServerConnectionAsync<F : HttpServerHandlerFactory> {
    factory: F,
}

impl<F : HttpServerHandlerFactory> HttpServerConnectionAsync<F> {
    pub fn new(lh: LoopHandle, socket: TcpStream, factory : F) -> HttpFuture<()> {
        let (to_write_tx, to_write_rx) = lh.clone().channel();

        let to_write_rx = future_flatten_to_stream(to_write_rx).map_err(HttpError::from).boxed();

        let handshake = server_handshake(socket);

        let run = handshake.and_then(move |socket| {
            let (read, write) = TaskIo::new(socket).split();

            let inner = TaskRcMut::new(ServerInner {
                conn: HttpConnection::new(HttpScheme::Http),
                to_write_tx: to_write_tx.clone(),
                session_state: GrpcHttpServerSessionState {
                    streams: HashMap::new(),
                    factory: factory,
                },
            });

            let run_write = ServerWriteLoop { write: write, inner: inner.clone() }.run(to_write_rx);
            let run_read = ServerReadLoop { read: read, inner: inner.clone() }.run();

            run_write.join(run_read).map(|_| ())
        });

        run.boxed()
    }
}
