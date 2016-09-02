use std::marker;

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
use tokio_core::io::TaskIo;
use tokio_core::io::TaskIoRead;
use tokio_core::io::TaskIoWrite;
use tokio_core::LoopHandle;

use error::*;
use futures_misc::*;

use solicit_async::*;

struct Inner {
    host: String,
    conn: HttpConnection,
}

struct HttpConnectionAsync<H : ResponseHandler> {
    _marker: marker::PhantomData<H>,
    inner: TaskRcMut<Inner>,
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

}

enum ToWriteMessage<H : ResponseHandler> {
    Start(StartRequestMessage<H>),
    FromRead(ReadToWriteMessage),
}

fn run_write<H : ResponseHandler>(
    write: TaskIoWrite<TcpStream>,
    inner: TaskRcMut<Inner>,
    requests: tokio_core::Receiver<ToWriteMessage<H>>)
        -> HttpFuture<()>
{
    let requests = requests.map_err(HttpError::from);
    requests.fold((), |_, _| {
        futures::finished::<_, HttpError>(())
    }).boxed()
}

struct ReadLoop {
    read: TaskIoRead<TcpStream>,
    inner: TaskRcMut<Inner>,
}

impl ReadLoop {
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
            });

            let write_inner = inner.clone();
            let run_write = call_rx.map_err(HttpError::from).and_then(move |call_rx| run_write(write, write_inner, call_rx));
            let run_read = ReadLoop { read: read, inner: inner.clone() }.run();

            let c = HttpConnectionAsync {
                _marker: marker::PhantomData,
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
