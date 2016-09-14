use std::net::SocketAddr;
use std::collections::HashMap;

use solicit::http::session::StreamState;
use solicit::http::connection::HttpConnection;
use solicit::http::connection::EndStream;
use solicit::http::connection::SendFrame;
use solicit::http::frame::*;
use solicit::http::StreamId;
use solicit::http::HttpScheme;
use solicit::http::HttpError;
use solicit::http::StaticHeader;
use hpack;

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

struct GrpcHttpClientStream<H : HttpClientResponseHandler> {
    state: StreamState,
    response_handler: Option<H>,
}

impl<H : HttpClientResponseHandler> GrpcHttpClientStream<H> {

    fn _close(&mut self) {
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
        self.response_handler.take();
    }

    fn _is_closed(&self) -> bool {
        self.state() == StreamState::Closed
    }

    fn _is_closed_local(&self) -> bool {
        match self.state() {
            StreamState::HalfClosedLocal | StreamState::Closed => true,
            _ => false,
        }
    }

    fn _is_closed_remote(&self) -> bool {
        match self.state() {
            StreamState::HalfClosedRemote | StreamState::Closed => true,
            _ => false,
        }
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
    decoder: hpack::Decoder<'static>,
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

    fn remove_stream(&mut self, stream_id: StreamId) -> Option<GrpcHttpClientStream<H>> {
        self.streams.remove(&stream_id)
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
    fn recv_raw_frame(self) -> HttpFuture<(Self, RawFrame<'static>)> {
        let ClientReadLoop { read, inner } = self;
        Box::new(recv_raw_frame(read)
            .map(|(read, frame)| (ClientReadLoop { read: read, inner: inner}, frame))
            .map_err(HttpError::from))
    }


    fn process_data_frame(self, frame: DataFrame) -> HttpFuture<Self> {
        self.inner.with(|inner: &mut ClientInner<H>| {
            let mut stream: &mut GrpcHttpClientStream<H> = match inner.session_state.get_stream_mut(frame.get_stream_id()) {
                None => {
                    // TODO(mlalic): This can currently indicate two things:
                    //                 1) the stream was idle => PROTOCOL_ERROR
                    //                 2) the stream was closed => STREAM_CLOSED (stream error)
                    return;
                }
                Some(stream) => stream,
            };

            if let Some(ref mut response_handler) = stream.response_handler {
                response_handler.part(HttpStreamPart {
                    content: HttpStreamPartContent::Data(frame.data.clone().into_owned()),
                    last: frame.is_end_of_stream(),
                });
            }

            if frame.is_end_of_stream() {
                stream.close_remote();
                // TODO: GC streams
            }
        });
        Box::new(futures::finished(self))
    }

    fn process_headers_frame(self, frame: HeadersFrame) -> HttpFuture<Self> {
        self.inner.with(|inner: &mut ClientInner<H>| {
            let headers = inner.session_state.decoder
                                   .decode(&frame.header_fragment())
                                   .map_err(HttpError::CompressionError).unwrap();
            let headers: Vec<StaticHeader> = headers.into_iter().map(|h| h.into()).collect();

            let mut stream: &mut GrpcHttpClientStream<H> = match inner.session_state.get_stream_mut(frame.get_stream_id()) {
                None => {
                    // TODO(mlalic): This means that the server's header is not associated to any
                    //               request made by the client nor any server-initiated stream (pushed)
                    return;
                }
                Some(stream) => stream,
            };
            // TODO: hack
            if headers.len() != 0 {

                if let Some(ref mut response_handler) = stream.response_handler {
                    response_handler.part(HttpStreamPart {
                        content: HttpStreamPartContent::Headers(headers),
                        last: frame.is_end_of_stream(),
                    });
                }
            }

            if frame.is_end_of_stream() {
                stream.close_remote();
                // TODO: GC streams
            }
        });
        Box::new(futures::finished(self))
    }

    fn process_rst_stream_frame(self, frame: RstStreamFrame) -> HttpFuture<Self> {
        self.inner.with(move |inner: &mut ClientInner<H>| {
            // TODO: check actually removed
            inner.session_state.remove_stream(frame.get_stream_id());
        });

        Box::new(futures::finished(self))
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
    fn read_process_frame(self) -> HttpFuture<Self> {
        Box::new(self.recv_raw_frame()
            .and_then(move |(rl, frame)| rl.process_raw_frame(frame)))
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
                    decoder: hpack::Decoder::new(),
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
