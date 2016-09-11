use std::marker;
use std::collections::HashMap;

use solicit::http::session::StreamState;
use solicit::http::session::Session;
use solicit::http::connection::HttpConnection;
use solicit::http::connection::EndStream;
use solicit::http::connection::SendFrame;
use solicit::http::frame::SettingsFrame;
use solicit::http::frame::RawFrame;
use solicit::http::frame::HttpSetting;
use solicit::http::frame::PingFrame;
use solicit::http::frame::FrameIR;
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
use tokio_core::net::TcpStream;
use tokio_core::io as tokio_io;
use tokio_core::io::Io;
use tokio_core::io::ReadHalf;
use tokio_core::io::WriteHalf;
use tokio_core::reactor;

use futures_misc::*;

use solicit_async::*;
use solicit_misc::*;
use http_common::*;


pub enum AfterHeaders {
    DataChunk(Vec<u8>, HttpFutureSend<AfterHeaders>),
    Trailers(Vec<StaticHeader>),
}

pub struct ResponseHeaders {
    pub headers: Vec<StaticHeader>,
    pub after: HttpFutureSend<AfterHeaders>,
}


pub trait HttpServerHandlerFactory: Send + 'static {
    fn new_request(&mut self, req: HttpStreamStreamSend) -> HttpStreamStreamSend;
}

struct GrpcHttpServerStream<F : HttpServerHandlerFactory> {
    state: StreamState,
    request_handler: Option<tokio_core::channel::Sender<ResultOrEof<HttpStreamPart, HttpError>>>,
    _marker: marker::PhantomData<F>,
}

impl<F : HttpServerHandlerFactory> GrpcHttpServerStream<F> {
    fn on_rst_stream(&mut self, _error_code: ErrorCode) {
        self.close();
    }

    fn close(&mut self) {
        self.set_state(StreamState::Closed);
    }

    fn _close_local(&mut self) {
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

    fn _is_closed_local(&self) -> bool {
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
        if let Some(ref mut sender) = self.request_handler {
            // ignore error
            sender.send(ResultOrEof::Item(HttpStreamPart::Data(data.to_owned()))).ok();
        }
    }

    fn set_headers<'n, 'v>(&mut self, headers: Vec<Header<'n, 'v>>) {
        let headers = headers.into_iter().map(|h| Header::new(h.name().to_owned(), h.value().to_owned())).collect();
        if let Some(ref mut sender) = self.request_handler {
            // ignore error
            sender.send(ResultOrEof::Item(HttpStreamPart::Headers(headers))).ok();
        }
    }

    fn set_state(&mut self, state: StreamState) {
        self.state = state;
        if self.is_closed_remote() {
            if let Some(sender) = self.request_handler.take() {
                // ignore error
                sender.send(ResultOrEof::Eof).ok();
            }
        }
    }

    fn state(&self) -> StreamState {
        self.state
    }

}

struct GrpcHttpServerSessionState<F : HttpServerHandlerFactory> {
    factory: F,
    streams: HashMap<StreamId, GrpcHttpServerStream<F>>,
    to_write_tx: tokio_core::channel::Sender<ServerToWriteMessage<F>>,
    loop_handle: reactor::Handle,
}

impl<F : HttpServerHandlerFactory> GrpcHttpServerSessionState<F> {
    fn insert_stream(&mut self, stream_id: StreamId, stream: GrpcHttpServerStream<F>) -> &mut GrpcHttpServerStream<F> {
        self.streams.insert(stream_id, stream);
        self.streams.get_mut(&stream_id).unwrap()
    }

    fn _get_stream_ref(&self, stream_id: StreamId) -> Option<&GrpcHttpServerStream<F>> {
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
    _Empty,
    _Headers(Vec<StaticHeader>),
    _Chunk(Vec<u8>),
    _Trailers(Vec<StaticHeader>),
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

        let (req_tx, req_rx) = tokio_core::channel::channel(&self.state.loop_handle).unwrap();

        let req_rx = req_rx.map_err(HttpError::from);
        let req_rx = stream_with_eof_and_error(req_rx);

        let response = self.state.factory.new_request(Box::new(req_rx));

        {
            let to_write_tx = self.state.to_write_tx.clone();
            let to_write_tx2 = to_write_tx.clone();

            let process_response = response.for_each(move |part: HttpStreamPart| {
                to_write_tx.send(ServerToWriteMessage::ResponsePart(stream_id, part)).unwrap();
                Ok(())
            }).and_then(move |()| {
                to_write_tx2.send(ServerToWriteMessage::ResponseStreamEnd(stream_id)).unwrap();
                Ok(())
            }).map_err(|e| panic!("{:?}", e)); // TODO: handle

            self.state.loop_handle.spawn(process_response);
        }

        // New stream initiated by the client
        let stream = GrpcHttpServerStream {
            state: StreamState::Open,
            request_handler: Some(req_tx),
            _marker: marker::PhantomData,
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
}

struct ServerReadLoop<F>
    where F : HttpServerHandlerFactory
{
    read: ReadHalf<TcpStream>,
    inner: TaskRcMut<ServerInner<F>>,
}


struct ServerReadToWriteMessage {
    buf: Vec<u8>,
}

enum ServerToWriteMessage<F : HttpServerHandlerFactory> {
    _Dummy(F),
    FromRead(ServerReadToWriteMessage),
    ResponsePart(StreamId, HttpStreamPart),
    ResponseStreamEnd(StreamId),
}


impl<F : HttpServerHandlerFactory> ServerReadLoop<F> {
    fn recv_raw_frame(self) -> HttpFuture<(Self, RawFrame<'static>)> {
        let ServerReadLoop { read, inner } = self;
        Box::new(recv_raw_frame(read)
            .map(|(read, frame)| (ServerReadLoop { read: read, inner: inner}, frame))
            .map_err(HttpError::from))
    }

    fn read_process_frame(self) -> HttpFuture<Self> {
        Box::new(self.recv_raw_frame()
            .and_then(move |(rl, frame)| rl.process_raw_frame(frame)))
    }

    fn process_settings_global(self, _frame: SettingsFrame) -> HttpFuture<Self> {
        // TODO: apply settings
        Box::new(futures::finished(self))
    }

    fn send_frame<R : FrameIR>(self, frame: R) -> HttpFuture<Self> {
        self.inner.with(|inner: &mut ServerInner<F>| {
            let mut send_buf = VecSendFrame(Vec::new());
            send_buf.send_frame(frame).unwrap();
            inner.session_state.to_write_tx.send(ServerToWriteMessage::FromRead(ServerReadToWriteMessage { buf: send_buf.0 }))
                .expect("read to write");
        });

        Box::new(futures::finished(self))
    }

    fn process_ping(self, frame: PingFrame) -> HttpFuture<Self> {
        if frame.is_ack() {
            Box::new(futures::finished(self))
        } else {
            self.send_frame(PingFrame::new_ack(frame.opaque_data()))
        }
    }

    fn process_special_frame(self, frame: HttpFrameConn) -> HttpFuture<Self> {
        match frame {
            HttpFrameConn::Settings(f) => self.process_settings_global(f),
            HttpFrameConn::Ping(f) => self.process_ping(f),
            HttpFrameConn::Goaway(_f) => panic!("TODO"),
            HttpFrameConn::WindowUpdate(_f) => panic!("TODO"),
        }
    }

    fn process_stream_frame(self, frame: HttpFrameStream) -> HttpFuture<Self> {
        let last = self.inner.with(|inner: &mut ServerInner<F>| {
            let mut send = VecSendFrame(Vec::new());
            let last = {
                let mut session = MyServerSession::new(&mut inner.session_state, &mut send);
                inner.conn.handle_frame(frame.into_frame(), &mut session)
                    .unwrap();
                session.last.take()
            };

            if !send.0.is_empty() {
                inner.session_state.to_write_tx.send(ServerToWriteMessage::FromRead(ServerReadToWriteMessage { buf: send.0 }))
                    .expect("read to write");
            }

            if let &Some((stream_id, _)) = &last {
                let mut closed = false;
                if let Some(stream) = inner.session_state.get_stream_mut(stream_id) {
                    closed = stream.is_closed();
                }
                if closed {
                   inner.session_state.remove_stream(stream_id);
                }
            }

            last
        });

        if let Some((_stream_id, _last_chunk)) = last {
            unimplemented!()
        } else {
            Box::new(futures::finished(self))
        }
    }

    fn process_raw_frame(self, raw_frame: RawFrame) -> HttpFuture<Self> {
        let frame = HttpFrameClassified::from_raw(&raw_frame).unwrap();
        match frame {
            HttpFrameClassified::Conn(f) => self.process_special_frame(f),
            HttpFrameClassified::Stream(f) => self.process_stream_frame(f),
            HttpFrameClassified::Unknown(_f) => panic!("TODO"),
        }
    }

    fn run(self) -> HttpFuture<()> {
        let stream = stream_repeat(());

        let future = stream.fold(self, |rl, _| {
            rl.read_process_frame()
        });

        Box::new(future.map(|_| ()))
    }
}

struct ServerWriteLoop<F>
    where F : HttpServerHandlerFactory
{
    write: WriteHalf<TcpStream>,
    inner: TaskRcMut<ServerInner<F>>,
}

impl<F : HttpServerHandlerFactory> ServerWriteLoop<F> {
    fn _loop_handle(&self) -> reactor::Handle {
        self.inner.with(move |inner: &mut ServerInner<F>| inner.session_state.loop_handle.clone())
    }

    fn _to_write_tx(&self) -> tokio_core::channel::Sender<ServerToWriteMessage<F>> {
        self.inner.with(move |inner: &mut ServerInner<F>| inner.session_state.to_write_tx.clone())
    }

    fn write_all(self, buf: Vec<u8>) -> HttpFuture<Self> {
        let ServerWriteLoop { write, inner } = self;

        Box::new(tokio_io::write_all(write, buf)
            .map(move |(write, _)| ServerWriteLoop { write: write, inner: inner })
            .map_err(HttpError::from))
    }

    fn process_from_read(self, message: ServerReadToWriteMessage) -> HttpFuture<Self> {
        self.write_all(message.buf)
    }

    fn process_response_part(self, stream_id: StreamId, part: HttpStreamPart) -> HttpFuture<Self> {
        let send_buf = self.inner.with(move |inner: &mut ServerInner<F>| {
            inner.conn.send_part_to_vec(stream_id, part, EndStream::No).unwrap()
        });
        self.write_all(send_buf)
    }

    fn process_response_end(self, stream_id: StreamId) -> HttpFuture<Self> {
        println!("http server: process_response_end");
        let send_buf = self.inner.with(move |inner: &mut ServerInner<F>| {
            inner.conn.send_end_of_stream_to_vec(stream_id).unwrap()
        });
        self.write_all(send_buf)
    }

    fn process_message(self, message: ServerToWriteMessage<F>) -> HttpFuture<Self> {
        match message {
            ServerToWriteMessage::FromRead(from_read) => self.process_from_read(from_read),
            ServerToWriteMessage::ResponsePart(stream_id, response) => self.process_response_part(stream_id, response),
            ServerToWriteMessage::ResponseStreamEnd(stream_id) => self.process_response_end(stream_id),
            ServerToWriteMessage::_Dummy(..) => panic!(),
        }
    }

    fn run(self, requests: HttpStream<ServerToWriteMessage<F>>) -> HttpFuture<()> {
        let requests = requests.map_err(HttpError::from);
        Box::new(requests
            .fold(self, move |wl, message: ServerToWriteMessage<F>| {
                wl.process_message(message)
            })
            .map(|_| ()))
    }
}




pub struct HttpServerConnectionAsync<F : HttpServerHandlerFactory> {
    _marker: marker::PhantomData<F>,
}

impl<F : HttpServerHandlerFactory> HttpServerConnectionAsync<F> {
    pub fn new(lh: reactor::Handle, socket: TcpStream, factory : F) -> HttpFuture<()> {
        let (to_write_tx, to_write_rx) = tokio_core::channel::channel::<ServerToWriteMessage<F>>(&lh).unwrap();

        let handshake = server_handshake(socket);

        let run = handshake.and_then(move |socket| {
            let (read, write) = socket.split();

            let inner = TaskRcMut::new(ServerInner {
                conn: HttpConnection::new(HttpScheme::Http),
                session_state: GrpcHttpServerSessionState {
                    streams: HashMap::new(),
                    factory: factory,
                    to_write_tx: to_write_tx.clone(),
                    loop_handle: lh,
                },
            });

            let run_write = ServerWriteLoop { write: write, inner: inner.clone() }.run(Box::new(to_write_rx.map_err(HttpError::from)));
            let run_read = ServerReadLoop { read: read, inner: inner.clone() }.run();

            run_write.join(run_read).map(|_| ())
        });

        Box::new(run.then(|x| { println!("server: end"); x }))
    }
}
