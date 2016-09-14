use std::net::SocketAddr;
use std::collections::HashMap;

use solicit::http::session::StreamState;
use solicit::http::session::Session;
use solicit::http::connection::HttpConnection;
use solicit::http::connection::EndStream;
use solicit::http::connection::SendFrame;
use solicit::http::connection::HttpFrame;
use solicit::http::frame::*;
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


// TODO: make async
// TODO: replace with stream
pub trait HttpClientResponseHandler : Send + 'static {
    fn part(&mut self, part: HttpStreamPart) -> bool;
}

enum LastChunk {
    Empty,
    Headers(Vec<StaticHeader>),
    Chunk(Vec<u8>),
}

struct GrpcHttpClientStream<H : HttpClientResponseHandler> {
    state: StreamState,
    response_handler: Option<H>,
}

impl<H : HttpClientResponseHandler> GrpcHttpClientStream<H> {

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

    fn _close_remote(&mut self) {
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

    fn set_headers<'n, 'v>(&mut self, headers: Vec<Header<'n, 'v>>) -> LastChunk {
        let headers = headers.into_iter().map(|h| Header::new(h.name().to_owned(), h.value().to_owned())).collect();
        LastChunk::Headers(headers)
    }

    fn new_data_chunk(&mut self, data: &[u8]) -> LastChunk {
        LastChunk::Chunk(data.to_owned())
    }

    fn set_state(&mut self, state: StreamState) {
        self.state = state;
    }

    fn state(&self) -> StreamState {
        self.state
    }
}

struct GrpcHttpClientSessionState<H : HttpClientResponseHandler> {
    streams: HashMap<StreamId, GrpcHttpClientStream<H>>,
    next_stream_id: StreamId,
}

impl<H : HttpClientResponseHandler> GrpcHttpClientSessionState<H> {
    fn process_streams_after_handle_next_frame(&mut self, stream_id: StreamId, last_chunk: LastChunk, end_stream: EndStream) -> HttpFuture<()> {
        let last = end_stream == EndStream::Yes;
        let remove;
        if let Some(ref mut s) = self.streams.get_mut(&stream_id) {
            let mut ok = false;
            if let Some(ref mut response_handler) = s.response_handler {
                ok = match last_chunk {
                    LastChunk::Empty => true,
                    LastChunk::Chunk(chunk) => {
                        response_handler.part(HttpStreamPart {
                            content: HttpStreamPartContent::Data(chunk),
                            last: last,
                        })
                    }
                    LastChunk::Headers(headers) => {
                        response_handler.part(HttpStreamPart {
                            content: HttpStreamPartContent::Headers(headers),
                            last: last,
                        })
                    }
                };

            }

            if ok && s.is_closed_remote() {
            }

            if !ok {
                s.response_handler.take();
            }

            remove = s.is_closed();
        } else {
            return Box::new(futures::finished(()));
        }

        if remove {
            self.streams.remove(&stream_id);
        }

        Box::new(futures::finished(()))
    }
}

impl<H : HttpClientResponseHandler> GrpcHttpClientSessionState<H> {

    fn insert_stream(&mut self, stream: GrpcHttpClientStream<H>) -> StreamId {
        let id = self.next_stream_id;
        self.streams.insert(id, stream);
        self.next_stream_id += 2;
        id
    }

    fn _get_stream_ref(&self, stream_id: StreamId) -> Option<&GrpcHttpClientStream<H>> {
        self.streams.get(&stream_id)
    }

    fn get_stream_mut(&mut self, stream_id: StreamId) -> Option<&mut GrpcHttpClientStream<H>> {
        self.streams.get_mut(&stream_id)
    }

    fn _remove_stream(&mut self, stream_id: StreamId) -> Option<GrpcHttpClientStream<H>> {
        self.streams.remove(&stream_id)
    }
}

struct MyClientSession<'a, H, S>
    where
        H : HttpClientResponseHandler,
        S : SendFrame + 'a,
{
    state: &'a mut GrpcHttpClientSessionState<H>,
    sender: &'a mut S,
    last: Option<(StreamId, LastChunk)>,
}

impl<'a, H, S> MyClientSession<'a, H, S>
    where
        H : HttpClientResponseHandler,
        S : SendFrame + 'a,
{
    fn new(state: &'a mut GrpcHttpClientSessionState<H>, sender: &'a mut S) -> MyClientSession<'a, H, S> {
        MyClientSession {
            state: state,
            sender: sender,
            last: None,
        }
    }
}

impl<'a, H, S> Session for MyClientSession<'a, H, S>
    where
        H : HttpClientResponseHandler,
        S : SendFrame + 'a,
{
    fn new_data_chunk(&mut self, stream_id: StreamId, data: &[u8], _conn: &mut HttpConnection) -> HttpResult<()> {
        println!("client: new_data_chunk: {}", data.len());
        let mut stream: &mut GrpcHttpClientStream<H> = match self.state.get_stream_mut(stream_id) {
            None => {
                // TODO(mlalic): This can currently indicate two things:
                //                 1) the stream was idle => PROTOCOL_ERROR
                //                 2) the stream was closed => STREAM_CLOSED (stream error)
                return Ok(());
            }
            Some(stream) => stream,
        };
        // Now let the stream handle the data chunk
        self.last = Some((stream_id, stream.new_data_chunk(data)));
        Ok(())
    }

    fn new_headers<'n, 'v>(&mut self, stream_id: StreamId, headers: Vec<Header<'n, 'v>>, _conn: &mut HttpConnection) -> HttpResult<()> {
        println!("client: new_headers: {}", headers.len());
        let mut stream: &mut GrpcHttpClientStream<H> = match self.state.get_stream_mut(stream_id) {
            None => {
                // TODO(mlalic): This means that the server's header is not associated to any
                //               request made by the client nor any server-initiated stream (pushed)
                return Ok(());
            }
            Some(stream) => stream,
        };
        // TODO: hack
        if headers.len() != 0 {
            // Now let the stream handle the headers
            self.last = Some((stream_id, stream.set_headers(headers)));
        }
        Ok(())
    }

    fn end_of_stream(&mut self, stream_id: StreamId, _conn: &mut HttpConnection) -> HttpResult<()> {
        let mut stream = match self.state.get_stream_mut(stream_id) {
            None => {
                return Ok(());
            }
            Some(stream) => stream,
        };
        // Since this implies that the server has closed the stream (i.e. provided a response), we
        // close the local end of the stream, as well as the remote one; there's no need to keep
        // sending out the request body if the server's decided that it doesn't want to see it.
        stream.close();
        if let None = self.last {
            self.last = Some((stream_id, LastChunk::Empty));
        }
        Ok(())
    }

    fn rst_stream(&mut self, stream_id: StreamId, error_code: ErrorCode, _conn: &mut HttpConnection) -> HttpResult<()> {
        self.state.get_stream_mut(stream_id).map(|stream| stream.on_rst_stream(error_code));
        self.last = Some((stream_id, LastChunk::Empty));
        Ok(())
    }

    fn new_settings(&mut self, _settings: Vec<HttpSetting>, conn: &mut HttpConnection) -> HttpResult<()> {
        conn.sender(self.sender).send_settings_ack()
    }

    fn on_ping(&mut self, ping: &PingFrame, conn: &mut HttpConnection) -> HttpResult<()> {
        conn.sender(self.sender).send_ping_ack(ping.opaque_data())
    }

    fn on_pong(&mut self, _ping: &PingFrame, _conn: &mut HttpConnection) -> HttpResult<()> {
        Ok(())
    }
}

struct ClientInner<H : HttpClientResponseHandler> {
    conn: HttpConnection,
    to_write_tx: tokio_core::channel::Sender<ClientToWriteMessage<H>>,
    session_state: GrpcHttpClientSessionState<H>,
}

pub struct HttpClientConnectionAsync<H : HttpClientResponseHandler> {
    call_tx: tokio_core::channel::Sender<ClientToWriteMessage<H>>,
}

unsafe impl <H : HttpClientResponseHandler> Sync for HttpClientConnectionAsync<H> {}

struct StartRequestMessage<H : HttpClientResponseHandler> {
    headers: Vec<StaticHeader>,
    body: HttpStreamSend<Vec<u8>>,
    response_handler: H,
}

struct BodyChunkMessage {
    stream_id: StreamId,
    chunk: Vec<u8>,
}

struct EndRequestMessage {
    stream_id: StreamId,
}

struct ClientReadToWriteMessage {
    buf: Vec<u8>,
}

enum ClientToWriteMessage<H : HttpClientResponseHandler> {
    Start(StartRequestMessage<H>),
    BodyChunk(BodyChunkMessage),
    End(EndRequestMessage),
    FromRead(ClientReadToWriteMessage),
}

struct ClientWriteLoop<H : HttpClientResponseHandler> {
    write: WriteHalf<TcpStream>,
    inner: TaskRcMut<ClientInner<H>>,
}

impl<H : HttpClientResponseHandler> ClientWriteLoop<H> {
    fn write_all(self, buf: Vec<u8>) -> HttpFuture<Self> {
        let ClientWriteLoop { write, inner } = self;

        Box::new(tokio_io::write_all(write, buf)
            .map(move |(write, _)| ClientWriteLoop { write: write, inner: inner })
            .map_err(HttpError::from))
    }

    fn process_from_read(self, message: ClientReadToWriteMessage) -> HttpFuture<Self> {
        self.write_all(message.buf)
    }

    fn process_start(self, start: StartRequestMessage<H>) -> HttpFuture<Self> {
        let StartRequestMessage { headers, body, response_handler } = start;
        let (buf, stream_id) = self.inner.with(move |inner: &mut ClientInner<H>| {

            let stream = GrpcHttpClientStream {
                state: StreamState::Open,
                response_handler: Some(response_handler),
            };
            let stream_id = inner.session_state.insert_stream(stream);

            let send_buf = inner.conn.send_headers_to_vec(stream_id, &headers, EndStream::No).unwrap();

            (send_buf, stream_id)
        });

        Box::new(self.write_all(buf)
            .and_then(move |wl: ClientWriteLoop<H>| {
                body.fold(wl, move |wl: ClientWriteLoop<H>, chunk| {
                    wl.inner.with(|inner: &mut ClientInner<H>| {
                        inner.to_write_tx.send(ClientToWriteMessage::BodyChunk(BodyChunkMessage {
                            stream_id: stream_id,
                            chunk: chunk,
                        })).unwrap()
                    });
                    futures::finished::<_, HttpError>(wl)
                })
            })
            .and_then(move |wl: ClientWriteLoop<H>| {
                wl.inner.with(|inner: &mut ClientInner<H>| {
                    inner.to_write_tx.send(ClientToWriteMessage::End(EndRequestMessage {
                        stream_id: stream_id,
                    })).unwrap()
                });
                futures::finished::<_, HttpError>(wl)
            }))
    }

    fn process_body_chunk(self, body_chunk: BodyChunkMessage) -> HttpFuture<Self> {
        println!("client: process request body chunk");

        let BodyChunkMessage { stream_id, chunk } = body_chunk;

        let buf = self.inner.with(move |inner: &mut ClientInner<H>| {
            {
                let _stream = inner.session_state.get_stream_mut(stream_id);
                // TODO: check stream state
            }

            inner.conn.send_data_frames_to_vec(stream_id, &chunk, EndStream::No).unwrap()
        });

        self.write_all(buf)
    }

    fn process_end(self, end: EndRequestMessage) -> HttpFuture<Self> {
        println!("client: process request end");
        let EndRequestMessage { stream_id } = end;

        let buf = self.inner.with(move |inner: &mut ClientInner<H>| {
            inner.conn.send_end_of_stream_to_vec(stream_id).unwrap()
        });

        self.write_all(buf)
    }

    fn process_message(self, message: ClientToWriteMessage<H>) -> HttpFuture<Self> {
        match message {
            ClientToWriteMessage::Start(start) => self.process_start(start),
            ClientToWriteMessage::BodyChunk(body_chunk) => self.process_body_chunk(body_chunk),
            ClientToWriteMessage::End(end) => self.process_end(end),
            ClientToWriteMessage::FromRead(from_read) => self.process_from_read(from_read),
        }
    }

    fn run(self, requests: HttpStreamSend<ClientToWriteMessage<H>>) -> HttpFuture<()> {
        let requests = requests.map_err(HttpError::from);
        Box::new(requests
            .fold(self, move |wl, message: ClientToWriteMessage<H>| {
                wl.process_message(message)
            })
            .map(|_| ()))
    }
}

struct ClientReadLoop<H : HttpClientResponseHandler> {
    read: ReadHalf<TcpStream>,
    inner: TaskRcMut<ClientInner<H>>,
}

impl<H : HttpClientResponseHandler> HttpReadLoop for ClientReadLoop<H> {
    fn process_data_frame(self, frame: DataFrame) -> HttpFuture<Self> {
        unimplemented!()
    }

    fn process_headers_frame(self, frame: HeadersFrame) -> HttpFuture<Self> {
        unimplemented!()
    }

    fn process_rst_stream_frame(self, frame: RstStreamFrame) -> HttpFuture<Self> {
        unimplemented!()
    }

    fn process_window_update_frame(self, _frame: WindowUpdateFrame) -> HttpFuture<Self> {
        // TODO
        Box::new(futures::finished(self))
    }

    fn process_settings_global(self, _frame: SettingsFrame) -> HttpFuture<Self> {
        // TODO: apply settings
        // TODO: send ack
        Box::new(futures::finished(self))
    }

    fn send_frame<R : FrameIR>(self, frame: R) -> HttpFuture<Self> {
        self.inner.with(|inner: &mut ClientInner<H>| {
            let mut send_buf = VecSendFrame(Vec::new());
            send_buf.send_frame(frame).unwrap();
            inner.to_write_tx.send(ClientToWriteMessage::FromRead(ClientReadToWriteMessage { buf: send_buf.0 }))
                .expect("read to write");
        });

        Box::new(futures::finished(self))
    }

    fn process_conn_window_update(self, _frame: WindowUpdateFrame) -> HttpFuture<Self> {
        // TODO
        Box::new(futures::finished(self))
    }
}

impl<H : HttpClientResponseHandler> ClientReadLoop<H> {
    fn recv_raw_frame(self) -> HttpFuture<(Self, RawFrame<'static>)> {
        let ClientReadLoop { read, inner } = self;
        Box::new(recv_raw_frame(read)
            .map(|(read, frame)| (ClientReadLoop { read: read, inner: inner}, frame))
            .map_err(HttpError::from))
    }

    fn process_streams_after_handle_next_frame(self, stream_id: StreamId, last_chunk: LastChunk, end_stream: EndStream) -> HttpFuture<Self> {
        let future = self.inner.with(|inner: &mut ClientInner<H>| {
            inner.session_state.process_streams_after_handle_next_frame(stream_id, last_chunk, end_stream)
        });

        Box::new(future.map(|_| self))
    }

    fn process_settings_global(self, _frame: SettingsFrame) -> HttpFuture<Self> {
        // TODO: apply settings
        // TODO: send ack
        Box::new(futures::finished(self))
    }

    fn process_conn_window_update(self, _frame: WindowUpdateFrame) -> HttpFuture<Self> {
        // TODO
        Box::new(futures::finished(self))
    }

    fn process_conn_frame(self, frame: HttpFrameConn) -> HttpFuture<Self> {
        match frame {
            HttpFrameConn::Settings(f) => self.process_settings_global(f),
            HttpFrameConn::Ping(f) => self.process_ping(f),
            HttpFrameConn::Goaway(_f) => panic!("TODO"),
            HttpFrameConn::WindowUpdate(f) => self.process_conn_window_update(f),
        }
    }

    fn process_stream_frame(self, frame: HttpFrameStream) -> HttpFuture<Self> {
        unimplemented!()
    }

    fn process_raw_frame_old(self, raw_frame: RawFrame) -> HttpFuture<Self> {
        if false {
            self.process_raw_frame(raw_frame)
        } else {
            println!("client: process_raw_frame");
            let last = self.inner.with(move |inner: &mut ClientInner<H>| {
                let mut send = VecSendFrame(Vec::new());
                let last = {
                    let mut session = MyClientSession::new(&mut inner.session_state, &mut send);
                    let frame = HttpFrame::from_raw(&raw_frame).unwrap();
                    let end_stream = match &frame {
                        &HttpFrame::DataFrame(ref data_frame) => data_frame.is_end_of_stream(),
                        &HttpFrame::HeadersFrame(ref headers_frame) => headers_frame.is_end_of_stream(),
                        _ => false,
                    };
                    inner.conn.handle_frame(frame, &mut session)
                        .unwrap();
                    session.last.take().map(|(stream_id, last_chunk)| (stream_id, last_chunk, if end_stream { EndStream::Yes } else { EndStream::No }))
                };

                if !send.0.is_empty() {
                    inner.to_write_tx.send(ClientToWriteMessage::FromRead(ClientReadToWriteMessage { buf: send.0 }))
                        .expect("read to write");
                }

                last
            });

            if let Some((stream_id, last_chunk, end_stream)) = last {
                self.process_streams_after_handle_next_frame(stream_id, last_chunk, end_stream)
            } else {
                Box::new(futures::finished(self))
            }
        }
    }

    fn read_process_frame(self) -> HttpFuture<Self> {
        Box::new(self.recv_raw_frame()
            .and_then(move |(rl, frame)| rl.process_raw_frame_old(frame)))
    }

    fn run(self) -> HttpFuture<()> {
        let stream = stream_repeat(());

        let future = stream.fold(self, |rl, _| {
            rl.read_process_frame()
        });

        Box::new(future.map(|_| ()))
    }
}

impl<H : HttpClientResponseHandler> HttpClientConnectionAsync<H> {
    pub fn new(lh: reactor::Handle, addr: &SocketAddr) -> (Self, HttpFuture<()>) {
        let (to_write_tx, to_write_rx) = tokio_core::channel::channel(&lh).unwrap();

        let to_write_rx = Box::new(to_write_rx.map_err(HttpError::from));

        let c = HttpClientConnectionAsync {
            call_tx: to_write_tx.clone(),
        };

        let handshake = connect_and_handshake(&lh, addr);
        let future = handshake.and_then(move |conn| {
            println!("client: handshake done");
            let (read, write) = conn.split();

            let inner = TaskRcMut::new(ClientInner {
                conn: HttpConnection::new(HttpScheme::Http),
                to_write_tx: to_write_tx.clone(),
                session_state: GrpcHttpClientSessionState {
                    next_stream_id: 1,
                    streams: HashMap::new(),
                }
            });

            let write_inner = inner.clone();
            let run_write = ClientWriteLoop { write: write, inner: write_inner }.run(to_write_rx);
            let run_read = ClientReadLoop { read: read, inner: inner.clone() }.run();

            run_write.join(run_read).map(|_| ())
        });

        (c, Box::new(future))
    }

    pub fn start_request(
        &self,
        headers: Vec<StaticHeader>,
        body: HttpStreamSend<Vec<u8>>,
        response_handler: H)
            -> HttpFutureSend<()>
    {
        self.call_tx.send(ClientToWriteMessage::Start(StartRequestMessage {
            headers: headers,
            body: body,
            response_handler: response_handler,
        })).unwrap();

        // should use bounded queue here, so future result
        Box::new(futures::finished(()))
    }
}
