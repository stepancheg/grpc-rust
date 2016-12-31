use std::net::SocketAddr;
use std::io;

use solicit::http::session::StreamState;
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
use tokio_core::io::WriteHalf;
use tokio_core::reactor;

use tokio_tls;

use futures_misc::*;

use solicit_async::*;
use solicit_misc::*;

use http_common::*;


struct GrpcHttpClientStream {
    common: GrpcHttpStreamCommon,
    response_handler: Option<tokio_core::channel::Sender<ResultOrEof<HttpStreamPart, HttpError>>>,
}

impl GrpcHttpStream for GrpcHttpClientStream {
    fn common(&self) -> &GrpcHttpStreamCommon {
        &self.common
    }

    fn common_mut(&mut self) -> &mut GrpcHttpStreamCommon {
        &mut self.common
    }

    fn new_data_chunk(&mut self, data: &[u8], last: bool) {
        if let Some(ref mut response_handler) = self.response_handler {
            response_handler.send(ResultOrEof::Item(HttpStreamPart {
                content: HttpStreamPartContent::Data(data.to_owned()),
                last: last,
            })).unwrap();
        }
    }

    fn close_remote(&mut self) {
        let next = match self.state() {
            StreamState::HalfClosedLocal => StreamState::Closed,
            _ => StreamState::HalfClosedRemote,
        };
        self.set_state(next);
        if let Some(response_handler) = self.response_handler.take() {
            response_handler.send(ResultOrEof::Eof).unwrap();
        }
    }
}

impl GrpcHttpClientStream {

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
        self.common.state = state;
    }

    fn state(&self) -> StreamState {
        self.common.state
    }
}

struct GrpcHttpClientSessionState {
    next_stream_id: StreamId,
    decoder: hpack::Decoder<'static>,
    loop_handle: reactor::Handle,
}

struct ClientInner {
    common: LoopInnerCommon<GrpcHttpClientStream>,
    to_write_tx: tokio_core::channel::Sender<ClientToWriteMessage>,
    session_state: GrpcHttpClientSessionState,
}

impl ClientInner {
    fn insert_stream(&mut self, stream: GrpcHttpClientStream) -> StreamId {
        let id = self.session_state.next_stream_id;
        if let Some(..) = self.common.streams.insert(id, stream) {
            panic!("inserted stream that already existed");
        }
        self.session_state.next_stream_id += 2;
        id
    }

    fn _get_stream_ref(&self, stream_id: StreamId) -> Option<&GrpcHttpClientStream> {
        self.common.streams.get(&stream_id)
    }
}

impl LoopInner for ClientInner {
    type LoopHttpStream = GrpcHttpClientStream;

    fn common(&mut self) -> &mut LoopInnerCommon<GrpcHttpClientStream> {
        &mut self.common
    }

    fn send_common(&mut self, message: CommonToWriteMessage) {
        self.to_write_tx.send(ClientToWriteMessage::Common(message))
            .expect("read to write common");
    }

    fn process_headers_frame(&mut self, frame: HeadersFrame) {
        let headers = self.session_state.decoder
                               .decode(&frame.header_fragment())
                               .map_err(HttpError::CompressionError).unwrap();
        let headers: Vec<StaticHeader> = headers.into_iter().map(|h| h.into()).collect();

        let mut stream: &mut GrpcHttpClientStream = match self.common.get_stream_mut(frame.get_stream_id()) {
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
                response_handler.send(ResultOrEof::Item(HttpStreamPart {
                    content: HttpStreamPartContent::Headers(headers),
                    last: frame.is_end_of_stream(),
                })).unwrap();
            }
        }
    }
}

pub struct HttpClientConnectionAsync {
    call_tx: tokio_core::channel::Sender<ClientToWriteMessage>,
    remote: reactor::Remote,
}

unsafe impl  Sync for HttpClientConnectionAsync {}

struct StartRequestMessage {
    headers: Vec<StaticHeader>,
    body: HttpStreamSend<Vec<u8>>,
    response_handler: tokio_core::channel::Sender<ResultOrEof<HttpStreamPart, HttpError>>,
}

struct BodyChunkMessage {
    stream_id: StreamId,
    chunk: Vec<u8>,
}

struct EndRequestMessage {
    stream_id: StreamId,
}

enum ClientToWriteMessage {
    Start(StartRequestMessage),
    BodyChunk(BodyChunkMessage),
    End(EndRequestMessage),
    Common(CommonToWriteMessage),
}

struct ClientWriteLoop<I : Io + Send + 'static> {
    write: WriteHalf<I>,
    inner: TaskRcMut<ClientInner>,
}

impl<I : Io + Send + 'static> WriteLoop for ClientWriteLoop<I> {
    type Inner = ClientInner;

    fn with_inner<G, R>(&self, f: G) -> R
        where G: FnOnce(&mut Self::Inner) -> R
    {
        self.inner.with(f)
    }

    fn write_all(self, buf: Vec<u8>) -> HttpFuture<Self> {
        let ClientWriteLoop { write, inner } = self;

        Box::new(tokio_io::write_all(write, buf)
            .map(move |(write, _)| ClientWriteLoop { write: write, inner: inner })
            .map_err(HttpError::from))
    }
}

impl<I : Io + Send + 'static> ClientWriteLoop<I> {
    fn process_start(self, start: StartRequestMessage) -> HttpFuture<Self> {
        let StartRequestMessage { headers, body, response_handler } = start;

        let stream_id = self.inner.with(move |inner: &mut ClientInner| {

            let mut stream = GrpcHttpClientStream {
                common: GrpcHttpStreamCommon::new(),
                response_handler: Some(response_handler),
            };

            stream.common.outgoing.push_back(HttpStreamPartContent::Headers(headers));

            inner.insert_stream(stream)
        });

        let to_write_tx_1 = self.inner.with(|inner| inner.to_write_tx.clone());
        let to_write_tx_2 = to_write_tx_1.clone();

        self.inner.with(|inner: &mut ClientInner| {
            let future = body
                .fold((), move |(), chunk| {
                    to_write_tx_1.send(ClientToWriteMessage::BodyChunk(BodyChunkMessage {
                        stream_id: stream_id,
                        chunk: chunk,
                    })).unwrap();
                    futures::finished::<_, HttpError>(())
                });
            let future = future
                .and_then(move |()| {
                    to_write_tx_2.send(ClientToWriteMessage::End(EndRequestMessage {
                        stream_id: stream_id,
                    })).unwrap();
                    futures::finished::<_, HttpError>(())
                });

            let future = future.map_err(|e| {
                warn!("{:?}", e);
                ()
            });

            inner.session_state.loop_handle.spawn(future);
        });

        self.send_outg_stream(stream_id)
    }

    fn process_body_chunk(self, body_chunk: BodyChunkMessage) -> HttpFuture<Self> {
        let BodyChunkMessage { stream_id, chunk } = body_chunk;

        self.inner.with(move |inner: &mut ClientInner| {
            let stream = inner.common.get_stream_mut(stream_id)
                .expect(&format!("stream not found: {}", stream_id));
            // TODO: check stream state

            stream.common.outgoing.push_front(HttpStreamPartContent::Data(chunk));
        });

        self.send_outg_stream(stream_id)
    }

    fn process_end(self, end: EndRequestMessage) -> HttpFuture<Self> {
        let EndRequestMessage { stream_id } = end;

        let buf = self.inner.with(move |inner: &mut ClientInner| {
            inner.common.conn.send_end_of_stream_to_vec(stream_id).unwrap()
        });

        self.write_all(buf)
    }

    fn process_message(self, message: ClientToWriteMessage) -> HttpFuture<Self> {
        match message {
            ClientToWriteMessage::Start(start) => self.process_start(start),
            ClientToWriteMessage::BodyChunk(body_chunk) => self.process_body_chunk(body_chunk),
            ClientToWriteMessage::End(end) => self.process_end(end),
            ClientToWriteMessage::Common(common) => self.process_common(common),
        }
    }

    fn run(self, requests: HttpStreamSend<ClientToWriteMessage>) -> HttpFuture<()> {
        let requests = requests.map_err(HttpError::from);
        Box::new(requests
            .fold(self, move |wl, message: ClientToWriteMessage| {
                wl.process_message(message)
            })
            .map(|_| ()))
    }
}

type ClientReadLoop<I> = HttpReadLoopData<I, ClientInner>;

impl HttpClientConnectionAsync {
    fn connected<I : Io + Send + 'static>(lh: reactor::Handle, connect: HttpFutureSend<I>) -> (Self, HttpFuture<()>) {
        let (to_write_tx, to_write_rx) = tokio_core::channel::channel(&lh).unwrap();

        let to_write_rx = Box::new(to_write_rx.map_err(HttpError::from));

        let c = HttpClientConnectionAsync {
            remote: lh.remote().clone(),
            call_tx: to_write_tx.clone(),
        };

        let handshake = connect.and_then(client_handshake);

        let future = handshake.and_then(move |conn| {
            trace!("handshake done");
            let (read, write) = conn.split();

            let inner = TaskRcMut::new(ClientInner {
                common: LoopInnerCommon::new(HttpScheme::Http),
                to_write_tx: to_write_tx.clone(),
                session_state: GrpcHttpClientSessionState {
                    next_stream_id: 1,
                    decoder: hpack::Decoder::new(),
                    loop_handle: lh,
                }
            });

            let write_inner = inner.clone();
            let run_write = ClientWriteLoop { write: write, inner: write_inner }.run(to_write_rx);
            let run_read = ClientReadLoop { read: read, inner: inner.clone() }.run();

            run_write.join(run_read).map(|_| ())
        });

        (c, Box::new(future))
    }

    pub fn new_plain(lh: reactor::Handle, addr: &SocketAddr) -> (Self, HttpFuture<()>) {
        let addr = addr.clone();

        let connect = TcpStream::connect(&addr, &lh)
            .map(move |c| { info!("connected to {}", addr); c })
            .map_err(|e| e.into());

        HttpClientConnectionAsync::connected(lh, Box::new(connect))
    }

    pub fn new_tls(lh: reactor::Handle, addr: &SocketAddr) -> (Self, HttpFuture<()>) {
        unimplemented!()
        /*
        let addr = addr.clone();

        let connect = TcpStream::connect(&addr, &lh)
            .map(move |c| { info!("connected to {}", addr); c })
            .map_err(|e| e.into());

        let tls_conn = connect.and_then(|conn| {
            tokio_tls::ClientContext::new()
                .unwrap()
                .handshake("localhost", conn)
        });

        let tls_conn = tls_conn.map_err(HttpError::from);

        HttpClientConnectionAsync::connected(lh, Box::new(tls_conn))
        */
    }

    pub fn start_request(
        &self,
        headers: Vec<StaticHeader>,
        body: HttpStreamSend<Vec<u8>>)
            -> HttpStreamStreamSend
    {
        let (tx, rx) = futures::oneshot();

        let call_tx = self.call_tx.clone();

        self.remote.spawn(move |handle| {
            let (req_tx, req_rx) = tokio_core::channel::channel(&handle).unwrap();

            call_tx.send(ClientToWriteMessage::Start(StartRequestMessage {
                headers: headers,
                body: body,
                response_handler: req_tx,
            })).unwrap();

            let req_rx = req_rx.map_err(HttpError::from);

            tx.complete(stream_with_eof_and_error(req_rx, || HttpError::from(io::Error::new(io::ErrorKind::Other, "unexpected eof"))));

            Ok(())
        });

        let rx = rx.map_err(|_| HttpError::from(io::Error::new(io::ErrorKind::Other, "oneshot canceled")));

        Box::new(rx.flatten_stream())
    }
}
