use std::marker;
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



struct GrpcHttp2ClientStream2 {
    state: StreamState,
}

impl solicit_Stream for GrpcHttp2ClientStream2 {
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
    streams: HashMap<StreamId, GrpcHttp2ClientStream2>,
    next_stream_id: StreamId,
    _phantom_data: marker::PhantomData<H>,
}

impl<H : ResponseHandler> SessionState for Inner<H> {
    type Stream = GrpcHttp2ClientStream2;

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
    fn iter(&mut self) -> StreamIter<GrpcHttp2ClientStream2> {
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
    call_tx: tokio_core::Sender<ToWriteMessage<H>>,
}

trait ResponseHandler : Send + 'static {
    fn headers(self) -> HttpFuture<Self>;
    fn body_chunk(self) -> HttpFuture<Self>;
    fn end(self) -> HttpFuture<()>;
}

struct StartRequestMessage<H : ResponseHandler> {
    headers: Vec<()>,
    body: HttpStream<Vec<u8>>,
    response_handler: H,
}

struct ReadToWriteMessage {
    buf: Vec<u8>,
}

enum ToWriteMessage<H : ResponseHandler> {
    Start(StartRequestMessage<H>),
    FromRead(ReadToWriteMessage),
}

struct WriteLoop<H : ResponseHandler> {
    write: TaskIoWrite<TcpStream>,
    inner: TaskRcMut<Inner<H>>,
}

impl<H : ResponseHandler> WriteLoop<H> {
    fn process_from_read(self, message: ReadToWriteMessage) -> HttpFuture<Self> {
        let WriteLoop { write, inner } = self;

        tokio_io::write_all(write, message.buf)
            .map(move |(write, _)| WriteLoop { write: write, inner: inner })
            .map_err(HttpError::from)
            .boxed()
    }

    fn process_start(self, start: StartRequestMessage<H>) -> HttpFuture<Self> {
        futures::finished(self).boxed()
    }

    fn process_message(self, message: ToWriteMessage<H>) -> HttpFuture<Self> {
        match message {
            ToWriteMessage::Start(start) => self.process_start(start),
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
                _phantom_data: marker::PhantomData,
            });

            let write_inner = inner.clone();
            let run_write = call_rx.map_err(HttpError::from).and_then(move |call_rx| WriteLoop { write: write, inner: write_inner }.run(call_rx));
            let run_read = ReadLoop { read: read, inner: inner.clone() }.run();

            let c = HttpConnectionAsync {
                inner: inner,
                call_tx: call_tx,
            };
            Ok((c, run_write.join(run_read).map(|_| ()).boxed()))
        }).boxed()
    }

    fn start_request(
        &self,
        headers: Vec<()>,
        body: HttpStream<Vec<u8>>,
        response_handler: H)
            -> HttpFuture<()>
    {
        self.call_tx.send(ToWriteMessage::Start(StartRequestMessage {
            headers: headers,
            body: body,
            response_handler: response_handler,
        }));
        futures::finished(()).boxed()
    }
}
