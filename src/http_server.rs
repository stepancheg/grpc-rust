use std::marker;
use std::collections::HashMap;
use std::io;

use solicit::http::session::StreamState;
use solicit::http::connection::HttpConnection;
use solicit::http::connection::SendFrame;
use solicit::http::frame::*;
use solicit::http::StreamId;
use solicit::http::HttpScheme;
use solicit::http::HttpError;
use solicit::http::Header;
use solicit::http::StaticHeader;
use solicit::http::INITIAL_CONNECTION_WINDOW_SIZE;

use futures;
use futures::Future;
use futures::stream::Stream;

use tokio_core;
use tokio_core::io as tokio_io;
use tokio_core::io::Io;
use tokio_core::io::WriteHalf;
use tokio_core::net::TcpStream;
use tokio_core::reactor;

use tokio_tls;

use futures_misc::*;

use solicit_async::*;
use solicit_misc::*;
use http_common::*;


struct GrpcHttpServerStream<F : HttpService> {
    common: GrpcHttpStreamCommon,
    request_handler: Option<tokio_core::channel::Sender<ResultOrEof<HttpStreamPart, HttpError>>>,
    _marker: marker::PhantomData<F>,
}

impl<F : HttpService> GrpcHttpStream for GrpcHttpServerStream<F> {
    fn common(&self) -> &GrpcHttpStreamCommon {
        &self.common
    }

    fn common_mut(&mut self) -> &mut GrpcHttpStreamCommon {
        &mut self.common
    }
}

impl<F : HttpService> GrpcHttpServerStream<F> {
    fn _close(&mut self) {
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

    fn new_data_chunk(&mut self, data: &[u8], last: bool) {
        if let Some(ref mut sender) = self.request_handler {
            let part = HttpStreamPart {
                content: HttpStreamPartContent::Data(data.to_owned()),
                last: last,
            };
            // ignore error
            sender.send(ResultOrEof::Item(part)).ok();
        }
    }

    fn set_headers<'n, 'v>(&mut self, headers: Vec<Header<'n, 'v>>, last: bool) {
        let headers = headers.into_iter().map(|h| Header::new(h.name().to_owned(), h.value().to_owned())).collect();
        if let Some(ref mut sender) = self.request_handler {
            let part = HttpStreamPart {
                content: HttpStreamPartContent::Headers(headers),
                last: last,
            };
            // ignore error
            sender.send(ResultOrEof::Item(part)).ok();
        }
    }

    fn set_state(&mut self, state: StreamState) {
        self.common.state = state;
        if self.is_closed_remote() {
            if let Some(sender) = self.request_handler.take() {
                // ignore error
                sender.send(ResultOrEof::Eof).ok();
            }
        }
    }

    fn state(&self) -> StreamState {
        self.common.state
    }

}

struct GrpcHttpServerSessionState<F : HttpService> {
    factory: F,
    streams: HashMap<StreamId, GrpcHttpServerStream<F>>,
    to_write_tx: tokio_core::channel::Sender<ServerToWriteMessage<F>>,
    loop_handle: reactor::Handle,
}

impl<F : HttpService> GrpcHttpServerSessionState<F> {
    fn _get_stream_ref(&self, stream_id: StreamId) -> Option<&GrpcHttpServerStream<F>> {
        self.streams.get(&stream_id)
    }

    fn get_stream_mut(&mut self, stream_id: StreamId) -> Option<&mut GrpcHttpServerStream<F>> {
        self.streams.get_mut(&stream_id)
    }

    fn remove_stream(&mut self, stream_id: StreamId) -> Option<GrpcHttpServerStream<F>> {
        debug!("remove stream: {}", stream_id);

        self.streams.remove(&stream_id)
    }

    fn remove_stream_if_closed(&mut self, stream_id: StreamId) {
        if self.get_stream_mut(stream_id).expect("unknown stream").is_closed() {
            self.remove_stream(stream_id);
        }
    }

    fn new_request(&mut self, stream_id: StreamId, headers: Vec<StaticHeader>)
        -> tokio_core::channel::Sender<ResultOrEof<HttpStreamPart, HttpError>>
    {
        let (req_tx, req_rx) = tokio_core::channel::channel(&self.loop_handle)
            .expect("failed to create a channel");

        let req_rx = req_rx.map_err(HttpError::from);
        let req_rx = stream_with_eof_and_error(req_rx, || HttpError::from(io::Error::new(io::ErrorKind::Other, "unexpected eof")));

        let response = self.factory.new_request(headers, Box::new(req_rx));

        {
            let to_write_tx = self.to_write_tx.clone();
            let to_write_tx2 = to_write_tx.clone();

            let process_response = response.for_each(move |part: HttpStreamPart| {
                // drop error if connection is closed
                if let Err(e) = to_write_tx.send(ServerToWriteMessage::ResponsePart(stream_id, part)) {
                    warn!("failed to write to channel, probably connection is closed: {}", e);
                }
                Ok(())
            }).and_then(move |()| {
                // drop error if connection is closed
                if let Err(e) = to_write_tx2.send(ServerToWriteMessage::ResponseStreamEnd(stream_id)) {
                    warn!("failed to write to channel, probably connection is closed: {}", e);
                }
                Ok(())
            }).map_err(|e| panic!("{:?}", e)); // TODO: handle

            self.loop_handle.spawn(process_response);
        }

        req_tx
    }

    fn new_stream(&mut self, stream_id: StreamId, headers: Vec<StaticHeader>) -> &mut GrpcHttpServerStream<F> {
        debug!("new stream: {}", stream_id);

        let req_tx = self.new_request(stream_id, headers);

        // New stream initiated by the client
        let stream = GrpcHttpServerStream {
            common: GrpcHttpStreamCommon::new(),
            request_handler: Some(req_tx),
            _marker: marker::PhantomData,
        };
        self.streams.insert(stream_id, stream);
        self.streams.get_mut(&stream_id).unwrap()
    }

    fn get_or_create_stream(&mut self, stream_id: StreamId, headers: Vec<StaticHeader>, last: bool) -> &mut GrpcHttpServerStream<F> {
        if self.get_stream_mut(stream_id).is_some() {
            // https://github.com/rust-lang/rust/issues/36403
            let stream = self.get_stream_mut(stream_id).unwrap();
            stream.set_headers(headers, last);
            stream
        } else {
            self.new_stream(stream_id, headers)
        }
    }

}


struct ServerInner<F : HttpService> {
    conn: HttpConnection,
    session_state: GrpcHttpServerSessionState<F>,
}

impl<F : HttpService> HttpReadLoopInner for ServerInner<F> {
    fn send_frame<R : FrameIR>(&mut self, frame: R) {
        let mut send_buf = VecSendFrame(Vec::new());
        send_buf.send_frame(frame).unwrap();
        self.session_state.to_write_tx.send(ServerToWriteMessage::FromRead(ServerReadToWriteMessage { buf: send_buf.0 }))
            .expect("read to write");
    }

    fn process_headers_frame(&mut self, frame: HeadersFrame) {
        let headers = self.conn.decoder
                               .decode(&frame.header_fragment())
                               .map_err(HttpError::CompressionError).unwrap();
        let headers = headers.into_iter().map(|h| h.into()).collect();

        let _stream = self.session_state
            .get_or_create_stream(frame.get_stream_id(), headers, frame.is_end_of_stream());

        // TODO: drop stream if closed on both ends
    }

    fn process_data_frame(&mut self, frame: DataFrame) {
        let stream_id = frame.get_stream_id();

        let (increment_conn, increment_stream) = {
            let stream = self.session_state.get_stream_mut(frame.get_stream_id()).expect("stream not found");

            self.conn.decrease_in_window(frame.payload_len())
                .expect("failed to decrease conn win");
            stream.common.in_window_size.try_decrease(frame.payload_len() as i32)
                .expect("failed to decrease stream win");

            let increment_conn =
                if self.conn.in_window_size() < INITIAL_CONNECTION_WINDOW_SIZE / 2 {
                    let increment = INITIAL_CONNECTION_WINDOW_SIZE as u32;
                    self.conn.in_window_size.try_increase(increment).expect("failed to increase");

                    Some(increment)
                } else {
                    None
                };

            let increment_stream =
                if stream.common.in_window_size.size() < INITIAL_CONNECTION_WINDOW_SIZE / 2 {
                    let increment = INITIAL_CONNECTION_WINDOW_SIZE as u32;
                    stream.common.in_window_size.try_increase(increment).expect("failed to increase");

                    Some(increment)
                } else {
                    None
                };

            stream.new_data_chunk(&frame.data.as_ref(), frame.is_end_of_stream());

            // TODO: drop stream if closed on both ends

            (increment_conn, increment_stream)
        };

        if let Some(increment_conn) = increment_conn {
            self.send_frame(WindowUpdateFrame::for_connection(increment_conn));
        }

        if let Some(increment_stream) = increment_stream {
            self.send_frame(WindowUpdateFrame::for_stream(stream_id, increment_stream));
        }
    }

    fn process_window_update_frame(&mut self, _frame: WindowUpdateFrame) {
        // TODO
    }

    fn process_settings_global(&mut self, _frame: SettingsFrame) {
        // TODO: apply settings
        // TODO: send ack
    }

    fn process_conn_window_update(&mut self, _frame: WindowUpdateFrame) {
        // TODO
    }

    fn close_remote(&mut self, stream_id: StreamId) {
        debug!("close remote: {}", stream_id);

        {
            let mut stream = self.session_state.get_stream_mut(stream_id).expect("stream not found");
            stream.close_remote();
        };
        self.session_state.remove_stream_if_closed(stream_id);
    }
}

type ServerReadLoop<F, I> = HttpReadLoopData<I, ServerInner<F>>;


struct ServerReadToWriteMessage {
    buf: Vec<u8>,
}

enum ServerToWriteMessage<F : HttpService> {
    _Dummy(F),
    FromRead(ServerReadToWriteMessage),
    ResponsePart(StreamId, HttpStreamPart),
    ResponseStreamEnd(StreamId),
}

struct ServerWriteLoop<F, I>
    where
        F : HttpService,
        I : Io + 'static,
{
    write: WriteHalf<I>,
    inner: TaskRcMut<ServerInner<F>>,
}

impl<F : HttpService, I : Io> ServerWriteLoop<F, I> {
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
            let (buf, close) = {
                let stream = inner.session_state.get_stream_mut(stream_id);
                if let Some(stream) = stream {
                    if !stream.is_closed_local() {
                        if part.last {
                            stream.close_local();
                        }
                        (inner.conn.send_part_to_vec(stream_id, &part).unwrap(), part.last)
                    } else {
                        (Vec::new(), false)
                    }
                } else {
                    (Vec::new(), false)
                }
            };

            if close {
                inner.session_state.remove_stream_if_closed(stream_id);
            }

            buf
        });
        self.write_all(send_buf)
    }

    fn process_response_end(self, stream_id: StreamId) -> HttpFuture<Self> {
        let send_buf = self.inner.with(move |inner: &mut ServerInner<F>| {
            let (buf, close) = {
                let stream = inner.session_state.get_stream_mut(stream_id);
                if let Some(stream) = stream {
                    if !stream.is_closed_local() {
                        stream.close_local();
                        (inner.conn.send_end_of_stream_to_vec(stream_id).unwrap(), true)
                    } else {
                        (Vec::new(), false)
                    }
                } else {
                    (Vec::new(), false)
                }
            };

            if close {
                inner.session_state.remove_stream_if_closed(stream_id);
            }

            buf
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




pub struct HttpServerConnectionAsync {
}

impl HttpServerConnectionAsync {
    fn connected<F, I>(lh: &reactor::Handle, socket: HttpFutureSend<I>, factory : F) -> HttpFuture<()>
        where
            F : HttpService,
            I : Io + Send + 'static,
    {
        let lh = lh.clone();

        let (to_write_tx, to_write_rx) = tokio_core::channel::channel::<ServerToWriteMessage<F>>(&lh).unwrap();

        let handshake = socket.and_then(server_handshake);

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

        Box::new(run.then(|x| { info!("connection end: {:?}", x); x }))
    }

    pub fn new_plain<F>(lh: &reactor::Handle, socket: TcpStream, factory: F) -> HttpFuture<()>
        where
            F : HttpService,
    {
        HttpServerConnectionAsync::connected(lh, Box::new(futures::finished(socket)), factory)
    }

    pub fn new_tls<F>(lh: &reactor::Handle, socket: TcpStream, server_context: tokio_tls::ServerContext, factory: F) -> HttpFuture<()>
        where
            F : HttpService,
    {
        let stream = server_context.handshake(socket).map_err(HttpError::from);

        HttpServerConnectionAsync::connected(lh, Box::new(stream), factory)
    }

    pub fn new_plain_fn<F>(lh: &reactor::Handle, socket: TcpStream, f: F) -> HttpFuture<()>
        where
            F : Fn(Vec<StaticHeader>, HttpStreamStreamSend) -> HttpStreamStreamSend + Send + 'static,
    {
        struct HttpServiceFn<F>(F);

        impl<F> HttpService for HttpServiceFn<F>
            where F : Fn(Vec<StaticHeader>, HttpStreamStreamSend) -> HttpStreamStreamSend + Send + 'static
        {
            fn new_request(&mut self, headers: Vec<StaticHeader>, req: HttpStreamStreamSend) -> HttpStreamStreamSend {
                (self.0)(headers, req)
            }
        }

        HttpServerConnectionAsync::new_plain(lh, socket, HttpServiceFn(f))
    }

    pub fn new_tls_fn<F>(lh: &reactor::Handle, socket: TcpStream, server_context: tokio_tls::ServerContext, f: F) -> HttpFuture<()>
        where
            F : Fn(Vec<StaticHeader>, HttpStreamStreamSend) -> HttpStreamStreamSend + Send + 'static,
    {
        struct HttpServiceFn<F>(F);

        impl<F> HttpService for HttpServiceFn<F>
            where F : Fn(Vec<StaticHeader>, HttpStreamStreamSend) -> HttpStreamStreamSend + Send + 'static
        {
            fn new_request(&mut self, headers: Vec<StaticHeader>, req: HttpStreamStreamSend) -> HttpStreamStreamSend {
                (self.0)(headers, req)
            }
        }

        HttpServerConnectionAsync::new_tls(lh, socket, server_context, HttpServiceFn(f))
    }
}
