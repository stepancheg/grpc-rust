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


// TODO: make async
pub trait HttpClientResponseHandler: Send + 'static {
    // called once response headers received
    fn headers(&mut self, headers: Vec<StaticHeader>) -> bool;
    // called for each response data frame
    fn data_frame(&mut self, chunk: Vec<u8>) -> bool;
    // called for response trailers
    fn trailers(&mut self, headers: Vec<StaticHeader>) -> bool;
    // called at the end of reponse stream
    fn end(&mut self);
}

enum ResponseState {
    Init,
    GotHeaders,
    GotBodyChunk,
    GotTrailers,
}

enum LastChunk {
    Empty,
    Headers(Vec<StaticHeader>),
    Chunk(Vec<u8>),
    Trailers(Vec<StaticHeader>),
}

struct GrpcHttp2ClientStream2<H : HttpClientResponseHandler> {
    state: StreamState,
    response_state: ResponseState,
    last_chunk: LastChunk,
    response_handler: Option<H>,
}

impl<H : HttpClientResponseHandler> solicit_Stream for GrpcHttp2ClientStream2<H> {
    fn set_headers<'n, 'v>(&mut self, headers: Vec<Header<'n, 'v>>) {
        let headers = headers.into_iter().map(|h| Header::new(h.name().to_owned(), h.value().to_owned())).collect();
        let (last_chunk, response_state) = match self.response_state {
            ResponseState::Init => (LastChunk::Headers(headers), ResponseState::GotHeaders),
            ResponseState::GotHeaders => panic!(),
            ResponseState::GotBodyChunk => (LastChunk::Trailers(headers), ResponseState::GotTrailers),
            ResponseState::GotTrailers => panic!(),
        };
        self.last_chunk = last_chunk;
        self.response_state = response_state;
    }

    fn new_data_chunk(&mut self, data: &[u8]) {
        self.last_chunk = LastChunk::Chunk(data.to_owned());
        self.response_state = ResponseState::GotBodyChunk;
    }

    fn set_state(&mut self, state: StreamState) {
        self.state = state;
    }

    fn get_data_chunk(&mut self, buf: &mut [u8]) -> Result<StreamDataChunk, StreamDataError> {
        unimplemented!()
    }

    fn state(&self) -> StreamState {
        self.state
    }
}

struct MySessionState<H : HttpClientResponseHandler> {
    streams: HashMap<StreamId, GrpcHttp2ClientStream2<H>>,
    next_stream_id: StreamId,
}

impl<H : HttpClientResponseHandler> MySessionState<H> {
    fn process_streams_after_handle_next_frame(&mut self) {
        let mut remove_ids = Vec::new();

        for (id, s) in &mut self.streams {
            let mut ok = false;
            if let Some(ref mut response_handler) = s.response_handler {
                let last_chunk = mem::replace(&mut s.last_chunk, LastChunk::Empty);
                ok = match last_chunk {
                    LastChunk::Empty => true,
                    LastChunk::Chunk(chunk) => {
                        response_handler.data_frame(chunk)
                    }
                    LastChunk::Headers(headers) => {
                        response_handler.headers(headers)
                    }
                    LastChunk::Trailers(headers) => {
                        let r = response_handler.trailers(headers);
                        if r {
                            response_handler.end();
                        }
                        r
                    }
                };

            }

            if !ok {
                s.response_handler.take();
            }

            if s.is_closed() {
                remove_ids.push(*id);
            }
        }

        Vec::from_iter(remove_ids.into_iter().map(|i| self.remove_stream(i).unwrap()));
    }
}

impl<H : HttpClientResponseHandler> SessionState for MySessionState<H> {
    type Stream = GrpcHttp2ClientStream2<H>;

    fn insert_outgoing(&mut self, stream: Self::Stream) -> StreamId {
        let id = self.next_stream_id;
        self.streams.insert(id, stream);
        self.next_stream_id += 2;
        id
    }

    fn insert_incoming(&mut self, stream_id: StreamId, stream: Self::Stream) -> Result<(), ()> {
        panic!("unused on the client");
    }

    fn get_stream_ref(&self, stream_id: StreamId) -> Option<&Self::Stream> {
        self.streams.get(&stream_id)
    }

    fn get_stream_mut(&mut self, stream_id: StreamId) -> Option<&mut Self::Stream> {
        self.streams.get_mut(&stream_id)
    }

    fn remove_stream(&mut self, stream_id: StreamId) -> Option<Self::Stream> {
        self.streams.remove(&stream_id)
    }

    fn iter(&mut self) -> StreamIter<GrpcHttp2ClientStream2<H>> {
        // https://github.com/mlalic/solicit/pull/34
        unimplemented!()
    }

    /// Number of currently active streams
    fn len(&self) -> usize {
        self.streams.len()
    }
}

struct Inner<H : HttpClientResponseHandler> {
    conn: HttpConnection,
    call_tx: tokio_core::Sender<ToWriteMessage<H>>,
    session_state: MySessionState<H>,
}

pub struct HttpClientConnectionAsync<H : HttpClientResponseHandler> {
    call_tx: tokio_core::Sender<ToWriteMessage<H>>,
}

struct StartRequestMessage<H : HttpClientResponseHandler> {
    headers: Vec<StaticHeader>,
    body: HttpStream<Vec<u8>>,
    response_handler: H,
}

struct BodyChunkMessage {
    stream_id: StreamId,
    chunk: Vec<u8>,
}

struct EndRequestMessage {
    stream_id: StreamId,
}

struct ReadToWriteMessage {
    buf: Vec<u8>,
}

enum ToWriteMessage<H : HttpClientResponseHandler> {
    Start(StartRequestMessage<H>),
    BodyChunk(BodyChunkMessage),
    End(EndRequestMessage),
    FromRead(ReadToWriteMessage),
}

struct WriteLoop<H : HttpClientResponseHandler> {
    write: TaskIoWrite<TcpStream>,
    inner: TaskRcMut<Inner<H>>,
}

impl<H : HttpClientResponseHandler> WriteLoop<H> {
    fn write_all(self, buf: Vec<u8>) -> HttpFuture<Self> {
        let WriteLoop { write, inner } = self;

        tokio_io::write_all(write, buf)
            .map(move |(write, _)| WriteLoop { write: write, inner: inner })
            .map_err(HttpError::from)
            .boxed()
    }

    fn process_from_read(self, message: ReadToWriteMessage) -> HttpFuture<Self> {
        self.write_all(message.buf)
    }

    fn process_start(self, start: StartRequestMessage<H>) -> HttpFuture<Self> {
        let StartRequestMessage { headers, body, response_handler } = start;
        let (buf, stream_id) = self.inner.with(move |inner: &mut Inner<H>| {

            let stream = GrpcHttp2ClientStream2 {
                state: StreamState::Open,
                response_handler: Some(response_handler),
                last_chunk: LastChunk::Empty,
                response_state: ResponseState::Init,
            };
            let stream_id = inner.session_state.insert_outgoing(stream);

            let send_buf = {
                let mut send_buf = VecSendFrame(Vec::new());

                inner.conn.sender(&mut send_buf).send_headers(headers, stream_id, EndStream::No).unwrap();

                (send_buf.0, stream_id)
            };

            send_buf
        });

        self.write_all(buf)
            .and_then(move |wl: WriteLoop<H>| {
                body.fold(wl, move |wl: WriteLoop<H>, chunk| {
                    wl.inner.with(|inner: &mut Inner<H>| {
                        inner.call_tx.send(ToWriteMessage::BodyChunk(BodyChunkMessage {
                            stream_id: stream_id,
                            chunk: chunk,
                        }))
                    });
                    futures::finished::<_, HttpError>(wl)
                })
            })
            .and_then(move |wl: WriteLoop<H>| {
                wl.inner.with(|inner: &mut Inner<H>| {
                    inner.call_tx.send(ToWriteMessage::End(EndRequestMessage {
                        stream_id: stream_id,
                    }));
                });
                futures::finished::<_, HttpError>(wl)
            })
            .boxed()
    }

    fn process_body_chunk(self, body_chunk: BodyChunkMessage) -> HttpFuture<Self> {
        println!("client: process request body chunk");

        let BodyChunkMessage { stream_id, chunk } = body_chunk;

        let buf = self.inner.with(move |inner: &mut Inner<H>| {
            {
                let stream = inner.session_state.get_stream_mut(stream_id);
                // TODO: check stream state
            }

            let mut send_buf = VecSendFrame(Vec::new());

            let mut pos = 0;
            const MAX_CHUNK_SIZE: usize = 8 * 1024;
            while pos < chunk.len() {
                let end = cmp::min(chunk.len(), pos + MAX_CHUNK_SIZE);

                let data_chunk = DataChunk::new_borrowed(&chunk[pos..end], stream_id, EndStream::No);

                inner.conn.sender(&mut send_buf).send_data(data_chunk).unwrap();

                pos = end;
            }

            send_buf.0
        });

        self.write_all(buf)
    }

    fn process_end(self, end: EndRequestMessage) -> HttpFuture<Self> {
        println!("client: process request end");
        let EndRequestMessage { stream_id } = end;

        let buf = self.inner.with(move |inner: &mut Inner<H>| {
            let mut send_buf = VecSendFrame(Vec::new());

            let chunk = Vec::new();
            let data_chunk = DataChunk::new_borrowed(&chunk[..], stream_id, EndStream::Yes);

            inner.conn.sender(&mut send_buf).send_data(data_chunk).unwrap();

            send_buf.0
        });

        self.write_all(buf)
    }

    fn process_message(self, message: ToWriteMessage<H>) -> HttpFuture<Self> {
        match message {
            ToWriteMessage::Start(start) => self.process_start(start),
            ToWriteMessage::BodyChunk(body_chunk) => self.process_body_chunk(body_chunk),
            ToWriteMessage::End(end) => self.process_end(end),
            ToWriteMessage::FromRead(from_read) => self.process_from_read(from_read),
        }
    }

    fn run(self, requests: HttpStream<ToWriteMessage<H>>) -> HttpFuture<()> {
        let requests = requests.map_err(HttpError::from);
        requests
            .fold(self, move |wl, message: ToWriteMessage<H>| {
                wl.process_message(message)
            })
            .map(|_| ())
            .boxed()
    }
}

struct ReadLoop<H : HttpClientResponseHandler> {
    read: TaskIoRead<TcpStream>,
    inner: TaskRcMut<Inner<H>>,
}

impl<H : HttpClientResponseHandler> ReadLoop<H> {
    fn recv_raw_frame(self) -> HttpFuture<(Self, RawFrame<'static>)> {
        let ReadLoop { read, inner } = self;
        recv_raw_frame(read)
            .map(|(read, frame)| (ReadLoop { read: read, inner: inner}, frame))
            .map_err(HttpError::from)
            .boxed()
    }

    fn process_streams_after_handle_next_frame(self) -> HttpFuture<Self> {
        self.inner.with(|inner: &mut Inner<H>| {
            inner.session_state.process_streams_after_handle_next_frame();
        });

        futures::finished(self).boxed()
    }

    fn process_raw_frame(self, raw_frame: RawFrame) -> HttpFuture<Self> {
        self.inner.with(move |inner: &mut Inner<H>| {
            let mut send = VecSendFrame(Vec::new());
            {
                let mut session = ClientSession::new(&mut inner.session_state, &mut send);
                inner.conn.handle_next_frame(&mut OnceReceiveFrame::new(raw_frame), &mut session)
                    .unwrap();
            }

            if !send.0.is_empty() {
                inner.call_tx.send(ToWriteMessage::FromRead(ReadToWriteMessage { buf: send.0 }))
                    .expect("read to write");
            }
        });

        self.process_streams_after_handle_next_frame()
    }

    fn read_process_frame(self) -> HttpFuture<Self> {
        self.recv_raw_frame()
            .and_then(move |(rl, frame)| rl.process_raw_frame(frame))
            .boxed()
    }

    fn run(self) -> HttpFuture<()> {
        let stream = stream_repeat(());

        let future = stream.fold(self, |rl, _| {
            rl.read_process_frame()
        });

        future.map(|_| ()).boxed()
    }
}

impl<H : HttpClientResponseHandler> HttpClientConnectionAsync<H> {
    pub fn new(lh: LoopHandle, addr: &SocketAddr) -> (Self, HttpFuture<()>) {
        let (call_tx, call_rx) = lh.clone().channel();

        let call_rx = future_flatten_to_stream(call_rx).map_err(HttpError::from).boxed();

        let c = HttpClientConnectionAsync {
            call_tx: call_tx.clone(),
        };

        let handshake = connect_and_handshake(lh, addr);
        let future = handshake.and_then(move |conn| {
            let (read, write) = TaskIo::new(conn).split();

            let inner = TaskRcMut::new(Inner {
                conn: HttpConnection::new(HttpScheme::Http),
                call_tx: call_tx.clone(),
                session_state: MySessionState {
                    next_stream_id: 1,
                    streams: HashMap::new(),
                }
            });

            let write_inner = inner.clone();
            let run_write = WriteLoop { write: write, inner: write_inner }.run(call_rx);
            let run_read = ReadLoop { read: read, inner: inner.clone() }.run();

            run_write.join(run_read).map(|_| ())
        }).boxed();

        (c, future)
    }

    pub fn start_request(
        &self,
        headers: Vec<StaticHeader>,
        body: HttpStream<Vec<u8>>,
        response_handler: H)
            -> HttpFuture<()>
    {
        self.call_tx.send(ToWriteMessage::Start(StartRequestMessage {
            headers: headers,
            body: body,
            response_handler: response_handler,
        }));

        // should use bounded queue here, so future result
        futures::finished(()).boxed()
    }
}
