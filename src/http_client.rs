use std::marker;
use std::cmp;
use std::collections::HashMap;

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


trait ResponseHandler : Send + 'static {
    fn headers(self) -> HttpFuture<Self>;
    fn body_chunk(self) -> HttpFuture<Self>;
    fn end(self) -> HttpFuture<()>;
}

struct GrpcHttp2ClientStream2<H : ResponseHandler> {
    state: StreamState,
    response_handler: H,
}

impl<H : ResponseHandler> solicit_Stream for GrpcHttp2ClientStream2<H> {
    fn new_data_chunk(&mut self, data: &[u8]) {
    }

    fn set_headers<'n, 'v>(&mut self, headers: Vec<Header<'n, 'v>>) {
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

struct Inner<H : ResponseHandler> {
    host: String,
    conn: HttpConnection,
    streams: HashMap<StreamId, GrpcHttp2ClientStream2<H>>,
    next_stream_id: StreamId,
    call_tx: tokio_core::Sender<ToWriteMessage<H>>,
}

impl<H : ResponseHandler> SessionState for Inner<H> {
    type Stream = GrpcHttp2ClientStream2<H>;

    fn insert_outgoing(&mut self, stream: Self::Stream) -> StreamId {
        let id = self.next_stream_id;
        self.streams.insert(id, stream);
        self.next_stream_id += 2;
        id
    }

    fn insert_incoming(&mut self, stream_id: StreamId, stream: Self::Stream) -> Result<(), ()> {
        // TODO: assert parity
        // TODO: Assert that the stream IDs are monotonically increasing!
        self.streams.insert(stream_id, stream);
        Ok(())
    }

    #[inline]
    fn get_stream_ref(&self, stream_id: StreamId) -> Option<&Self::Stream> {
        self.streams.get(&stream_id)
    }

    #[inline]
    fn get_stream_mut(&mut self, stream_id: StreamId) -> Option<&mut Self::Stream> {
        self.streams.get_mut(&stream_id)
    }

    #[inline]
    fn remove_stream(&mut self, stream_id: StreamId) -> Option<Self::Stream> {
        self.streams.remove(&stream_id)
    }

    #[inline]
    fn iter(&mut self) -> StreamIter<GrpcHttp2ClientStream2<H>> {
        // https://github.com/mlalic/solicit/pull/34
        unimplemented!()
    }

    /// Number of currently active streams
    #[inline]
    fn len(&self) -> usize {
        self.streams.len()
    }
}

struct HttpConnectionAsync<H : ResponseHandler> {
    inner: TaskRcMut<Inner<H>>,
}

struct StartRequestMessage<H : ResponseHandler> {
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

enum ToWriteMessage<H : ResponseHandler> {
    Start(StartRequestMessage<H>),
    BodyChunk(BodyChunkMessage),
    End(EndRequestMessage),
    FromRead(ReadToWriteMessage),
}

struct WriteLoop<H : ResponseHandler> {
    write: TaskIoWrite<TcpStream>,
    inner: TaskRcMut<Inner<H>>,
}

impl<H : ResponseHandler> WriteLoop<H> {
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
                response_handler: response_handler,
            };
            let stream_id = inner.insert_outgoing(stream);

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
        let BodyChunkMessage { stream_id, chunk } = body_chunk;

        let buf = self.inner.with(move |inner: &mut Inner<H>| {
            {
                let stream = inner.get_stream_mut(stream_id);
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
        let EndRequestMessage { stream_id } = end;

        let buf = self.inner.with(move |inner: &mut Inner<H>| {
            let mut send_buf = VecSendFrame(Vec::new());

            let chunk = Vec::new();
            let data_chunk = DataChunk::new_borrowed(&chunk[..], stream_id, EndStream::Yes);

            inner.conn.sender(&mut send_buf).send_data(data_chunk).unwrap();

            send_buf.0
        });

        futures::finished(self).boxed()
    }

    fn process_message(self, message: ToWriteMessage<H>) -> HttpFuture<Self> {
        match message {
            ToWriteMessage::Start(start) => self.process_start(start),
            ToWriteMessage::BodyChunk(body_chunk) => self.process_body_chunk(body_chunk),
            ToWriteMessage::End(end) => self.process_end(end),
            ToWriteMessage::FromRead(from_read) => self.process_from_read(from_read),
        }
    }

    fn run(self, requests: tokio_core::Receiver<ToWriteMessage<H>>) -> HttpFuture<()> {
        let requests = requests.map_err(HttpError::from);
        requests
            .fold(self, move |wl, message: ToWriteMessage<H>| {
                wl.process_message(message)
            })
            .map(|_| ())
            .boxed()
    }
}

struct ReadLoop<H : ResponseHandler> {
    read: TaskIoRead<TcpStream>,
    inner: TaskRcMut<Inner<H>>,
}

impl<H : ResponseHandler> ReadLoop<H> {
    fn recv_raw_frame(self) -> HttpFuture<(Self, RawFrame<'static>)> {
        let ReadLoop { read, inner } = self;
        recv_raw_frame(read)
            .map(|(read, frame)| (ReadLoop { read: read, inner: inner}, frame))
            .map_err(HttpError::from)
            .boxed()
    }

    fn read_process_frame(self) -> HttpFuture<Self> {
        self.recv_raw_frame()
            .map(|(rl, frame)| rl)
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

impl<H : ResponseHandler> HttpConnectionAsync<H> {
    fn new(lh: LoopHandle, conn: TcpStream) -> HttpFuture<(Self, HttpFuture<()>)> {
        let (call_tx, call_rx) = lh.channel();

        let handshake = client_handshake(conn);
        handshake.and_then(move |conn| {
            let (read, write) = TaskIo::new(conn).split();

            let inner = TaskRcMut::new(Inner {
                host: "localhost".to_owned(), // TODO
                conn: HttpConnection::new(HttpScheme::Http),
                next_stream_id: 1,
                streams: HashMap::new(),
                call_tx: call_tx,
            });

            let write_inner = inner.clone();
            let run_write = call_rx.map_err(HttpError::from).and_then(move |call_rx| WriteLoop { write: write, inner: write_inner }.run(call_rx));
            let run_read = ReadLoop { read: read, inner: inner.clone() }.run();

            let c = HttpConnectionAsync {
                inner: inner,
            };
            Ok((c, run_write.join(run_read).map(|_| ()).boxed()))
        }).boxed()
    }

    fn start_request(
        &self,
        headers: Vec<StaticHeader>,
        body: HttpStream<Vec<u8>>,
        response_handler: H)
            -> HttpFuture<()>
    {
        self.inner.with(|inner: &mut Inner<H>| {
            inner.call_tx.send(ToWriteMessage::Start(StartRequestMessage {
                headers: headers,
                body: body,
                response_handler: response_handler,
            }));
        });
        futures::finished(()).boxed()
    }
}
