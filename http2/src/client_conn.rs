use std::net::SocketAddr;
use std::io;

use solicit::frame::*;
use solicit::StreamId;
use solicit::HttpScheme;
use solicit::HttpError;
use solicit::StaticHeader;
use hpack;

use futures;
use futures::Future;
use futures::stream::Stream;

use tokio_core::net::TcpStream;
use tokio_core::io::Io;
use tokio_core::reactor;

use futures_misc::*;

use solicit_async::*;

use http_common::*;


struct HttpClientStream {
    common: HttpStreamCommon,
    response_handler: Option<futures::sync::mpsc::UnboundedSender<ResultOrEof<HttpStreamPart, HttpError>>>,
}

impl HttpStream for HttpClientStream {
    fn common(&self) -> &HttpStreamCommon {
        &self.common
    }

    fn common_mut(&mut self) -> &mut HttpStreamCommon {
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

    fn closed_remote(&mut self) {
        if let Some(mut response_handler) = self.response_handler.take() {
            // it is OK to ignore error: handler may be already dead
            drop(response_handler.send(ResultOrEof::Eof));
        }
    }
}

impl HttpClientStream {
}

struct HttpClientSessionState {
    next_stream_id: StreamId,
    decoder: hpack::Decoder<'static>,
    loop_handle: reactor::Handle,
}

struct ClientInner {
    common: LoopInnerCommon<HttpClientStream>,
    to_write_tx: futures::sync::mpsc::UnboundedSender<ClientToWriteMessage>,
    session_state: HttpClientSessionState,
}

impl ClientInner {
    fn insert_stream(&mut self, stream: HttpClientStream) -> StreamId {
        let id = self.session_state.next_stream_id;
        if let Some(..) = self.common.streams.insert(id, stream) {
            panic!("inserted stream that already existed");
        }
        self.session_state.next_stream_id += 2;
        id
    }
}

impl LoopInner for ClientInner {
    type LoopHttpStream = HttpClientStream;

    fn common(&mut self) -> &mut LoopInnerCommon<HttpClientStream> {
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

        let mut stream: &mut HttpClientStream = match self.common.get_stream_mut(frame.get_stream_id()) {
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
    call_tx: futures::sync::mpsc::UnboundedSender<ClientToWriteMessage>,
    command_tx: futures::sync::mpsc::UnboundedSender<ClientCommandMessage>,
    _remote: reactor::Remote,
}

unsafe impl Sync for HttpClientConnectionAsync {}

struct StartRequestMessage {
    headers: Vec<StaticHeader>,
    body: HttpFutureStreamSend<Vec<u8>>,
    response_handler: futures::sync::mpsc::UnboundedSender<ResultOrEof<HttpStreamPart, HttpError>>,
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

enum ClientCommandMessage {
    DumpState(futures::sync::oneshot::Sender<ConnectionStateSnapshot>),
}


impl<I : Io + Send + 'static> ClientWriteLoop<I> {
    fn process_start(self, start: StartRequestMessage) -> HttpFuture<Self> {
        let StartRequestMessage { headers, body, response_handler } = start;

        let stream_id = self.inner.with(move |inner: &mut ClientInner| {

            let mut stream = HttpClientStream {
                common: HttpStreamCommon::new(),
                response_handler: Some(response_handler),
            };

            stream.common.outgoing.push_back(HttpStreamPartContent::Headers(headers));

            inner.insert_stream(stream)
        });

        let mut to_write_tx_1 = self.inner.with(|inner| inner.to_write_tx.clone());
        let mut to_write_tx_2 = to_write_tx_1.clone();

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

        self.inner.with(move |inner: &mut ClientInner| {
            let stream = inner.common.get_stream_mut(stream_id)
                .expect(&format!("stream not found: {}", stream_id));

            // TODO: check stream state
            stream.common.outgoing_end = true;
        });

        self.send_outg_stream(stream_id)
    }

    fn process_message(self, message: ClientToWriteMessage) -> HttpFuture<Self> {
        match message {
            ClientToWriteMessage::Start(start) => self.process_start(start),
            ClientToWriteMessage::BodyChunk(body_chunk) => self.process_body_chunk(body_chunk),
            ClientToWriteMessage::End(end) => self.process_end(end),
            ClientToWriteMessage::Common(common) => self.process_common(common),
        }
    }

    fn run(self, requests: HttpFutureStreamSend<ClientToWriteMessage>) -> HttpFuture<()> {
        let requests = requests.map_err(HttpError::from);
        Box::new(requests
            .fold(self, move |wl, message: ClientToWriteMessage| {
                wl.process_message(message)
            })
            .map(|_| ()))
    }
}

type ClientReadLoop<I> = ReadLoopData<I, ClientInner>;
type ClientWriteLoop<I> = WriteLoopData<I, ClientInner>;
type ClientCommandLoop = CommandLoopData<ClientInner>;


impl HttpClientConnectionAsync {
    fn connected<I : Io + Send + 'static>(lh: reactor::Handle, connect: HttpFutureSend<I>) -> (Self, HttpFuture<()>) {
        let (to_write_tx, to_write_rx) = futures::sync::mpsc::unbounded();
        let (command_tx, command_rx) = futures::sync::mpsc::unbounded();

        let to_write_rx = Box::new(to_write_rx.map_err(|()| HttpError::IoError(io::Error::new(io::ErrorKind::Other, "to_write"))));
        let command_rx = Box::new(command_rx.map_err(|()| HttpError::IoError(io::Error::new(io::ErrorKind::Other, "to_write"))));

        let c = HttpClientConnectionAsync {
            _remote: lh.remote().clone(),
            call_tx: to_write_tx.clone(),
            command_tx: command_tx,
        };

        let handshake = connect.and_then(client_handshake);

        let future = handshake.and_then(move |conn| {
            debug!("handshake done");
            let (read, write) = conn.split();

            let inner = TaskRcMut::new(ClientInner {
                common: LoopInnerCommon::new(HttpScheme::Http),
                to_write_tx: to_write_tx.clone(),
                session_state: HttpClientSessionState {
                    next_stream_id: 1,
                    decoder: hpack::Decoder::new(),
                    loop_handle: lh,
                }
            });

            let run_write = ClientWriteLoop { write: write, inner: inner.clone() }.run(to_write_rx);
            let run_read = ClientReadLoop { read: read, inner: inner.clone() }.run();
            let run_command = ClientCommandLoop { inner: inner.clone() }.run(command_rx);

            run_write.join(run_read).join(run_command).map(|_| ())
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

    pub fn new_tls(_lh: reactor::Handle, _addr: &SocketAddr) -> (Self, HttpFuture<()>) {
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
        body: HttpFutureStreamSend<Vec<u8>>)
            -> HttpPartFutureStreamSend
    {
        let (tx, rx) = futures::oneshot();

        let mut call_tx = self.call_tx.clone();

        let (req_tx, req_rx) = futures::sync::mpsc::unbounded();

        call_tx.send(ClientToWriteMessage::Start(StartRequestMessage {
            headers: headers,
            body: body,
            response_handler: req_tx,
        })).expect("send request to client");

        let req_rx = req_rx.map_err(|()| HttpError::from(io::Error::new(io::ErrorKind::Other, "req")));

        // TODO: future is no longer needed here
        tx.complete(stream_with_eof_and_error(req_rx, || HttpError::from(io::Error::new(io::ErrorKind::Other, "unexpected eof"))));

        let rx = rx.map_err(|_| HttpError::from(io::Error::new(io::ErrorKind::Other, "oneshot canceled")));

        Box::new(rx.flatten_stream())
    }

    /// For tests
    pub fn dump_state(&self) -> HttpFutureSend<ConnectionStateSnapshot> {
        let (tx, rx) = futures::oneshot();

        self.command_tx.clone().send(ClientCommandMessage::DumpState(tx))
            .expect("send request to dump state");

        let rx = rx.map_err(|_| HttpError::from(io::Error::new(io::ErrorKind::Other, "oneshot canceled")));

        Box::new(rx)
    }
}

impl ClientCommandLoop {
    fn process_dump_state(self, sender: futures::sync::oneshot::Sender<ConnectionStateSnapshot>) -> HttpFuture<Self> {
        // ignore send error, client might be already dead
        drop(sender.complete(self.inner.with(|inner| inner.common.dump_state())));
        Box::new(futures::finished(self))
    }

    fn process_message(self, message: ClientCommandMessage) -> HttpFuture<Self> {
        match message {
            ClientCommandMessage::DumpState(sender) => self.process_dump_state(sender),
        }
    }

    fn run(self, requests: HttpFutureStreamSend<ClientCommandMessage>) -> HttpFuture<()> {
        let requests = requests.map_err(HttpError::from);
        Box::new(requests
            .fold(self, move |l, message: ClientCommandMessage| {
                l.process_message(message)
            })
            .map(|_| ()))
    }
}
