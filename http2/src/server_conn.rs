use std::marker;
use std::io;
use std::sync::Arc;

use solicit::frame::*;
use solicit::StreamId;
use solicit::HttpScheme;
use solicit::HttpError;
use solicit::Header;

use bytes::Bytes;

use futures;
use futures::Future;
use futures::stream::Stream;

use tokio_core::io::Io;
use tokio_core::net::TcpStream;
use tokio_core::reactor;

//use tokio_tls;

use futures_misc::*;

use solicit_async::*;
use http_common::*;

use server_conf::*;


struct HttpServerStream<F : HttpService> {
    common: HttpStreamCommon,
    request_handler: Option<futures::sync::mpsc::UnboundedSender<ResultOrEof<HttpStreamPart, HttpError>>>,
    _marker: marker::PhantomData<F>,
}

impl<F : HttpService> HttpStream for HttpServerStream<F> {
    fn common(&self) -> &HttpStreamCommon {
        &self.common
    }

    fn common_mut(&mut self) -> &mut HttpStreamCommon {
        &mut self.common
    }

    fn new_data_chunk(&mut self, data: &[u8], last: bool) {
        if let Some(ref mut sender) = self.request_handler {
            let part = HttpStreamPart {
                content: HttpStreamPartContent::Data(Bytes::from(data)),
                last: last,
            };
            // ignore error
            sender.send(ResultOrEof::Item(part)).ok();
        }
    }

    fn closed_remote(&mut self) {
        if let Some(sender) = self.request_handler.take() {
            // ignore error
            sender.send(ResultOrEof::Eof).ok();
        }
    }
}

impl<F : HttpService> HttpServerStream<F> {
    fn set_headers(&mut self, headers: Vec<Header>, last: bool) {
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

}

struct HttpServerSessionState<F : HttpService> {
    factory: Arc<F>,
    to_write_tx: futures::sync::mpsc::UnboundedSender<ServerToWriteMessage<F>>,
    loop_handle: reactor::Handle,
}


struct ServerInner<F : HttpService> {
    common: LoopInnerCommon<HttpServerStream<F>>,
    session_state: HttpServerSessionState<F>,
}

impl<F : HttpService> ServerInner<F> {
    fn new_request(&mut self, stream_id: StreamId, headers: Vec<Header>)
        -> futures::sync::mpsc::UnboundedSender<ResultOrEof<HttpStreamPart, HttpError>>
    {
        let (req_tx, req_rx) = futures::sync::mpsc::unbounded();

        let req_rx = req_rx.map_err(|()| HttpError::from(io::Error::new(io::ErrorKind::Other, "req")));
        let req_rx = stream_with_eof_and_error(req_rx, || HttpError::from(io::Error::new(io::ErrorKind::Other, "unexpected eof")));

        let response = self.session_state.factory.new_request(headers, Box::new(req_rx));

        {
            let to_write_tx = self.session_state.to_write_tx.clone();
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

            self.session_state.loop_handle.spawn(process_response);
        }

        req_tx
    }

    fn new_stream(&mut self, stream_id: StreamId, headers: Vec<Header>) -> &mut HttpServerStream<F> {
        debug!("new stream: {}", stream_id);

        let req_tx = self.new_request(stream_id, headers);

        // New stream initiated by the client
        let stream = HttpServerStream {
            common: HttpStreamCommon::new(),
            request_handler: Some(req_tx),
            _marker: marker::PhantomData,
        };
        if let Some(..) = self.common.streams.insert(stream_id, stream) {
            panic!("inserted stream that already existed");
        }
        self.common.streams.get_mut(&stream_id).unwrap()
    }

    fn get_or_create_stream(&mut self, stream_id: StreamId, headers: Vec<Header>, last: bool) -> &mut HttpServerStream<F> {
        if self.common.get_stream_mut(stream_id).is_some() {
            // https://github.com/rust-lang/rust/issues/36403
            let stream = self.common.get_stream_mut(stream_id).unwrap();
            stream.set_headers(headers, last);
            stream
        } else {
            self.new_stream(stream_id, headers)
        }
    }
}

impl<F : HttpService> LoopInner for ServerInner<F> {
    type LoopHttpStream = HttpServerStream<F>;

    fn common(&mut self) -> &mut LoopInnerCommon<HttpServerStream<F>> {
        &mut self.common
    }

    fn send_common(&mut self, message: CommonToWriteMessage) {
        self.session_state.to_write_tx.send(ServerToWriteMessage::Common(message))
            .expect("read to write common");
    }

    fn process_headers_frame(&mut self, frame: HeadersFrame) {
        let headers = self.common.conn.decoder
                               .decode(&frame.header_fragment())
                               .map_err(HttpError::CompressionError).unwrap();
        let headers = headers.into_iter().map(|h| Header::new(h.0, h.1)).collect();

        let _stream = self.get_or_create_stream(
            frame.get_stream_id(),
            headers,
            frame.is_end_of_stream());

        // TODO: drop stream if closed on both ends
    }

}

type ServerReadLoop<F, I> = ReadLoopData<I, ServerInner<F>>;
type ServerWriteLoop<F, I> = WriteLoopData<I, ServerInner<F>>;
type ServerCommandLoop<F> = CommandLoopData<ServerInner<F>>;


enum ServerToWriteMessage<F : HttpService> {
    _Dummy(F),
    ResponsePart(StreamId, HttpStreamPart),
    // send when user provided handler completed the stream
    ResponseStreamEnd(StreamId),
    Common(CommonToWriteMessage),
}

enum ServerCommandMessage {
    DumpState(futures::sync::oneshot::Sender<ConnectionStateSnapshot>),
}


impl<F : HttpService, I : Io + Send> ServerWriteLoop<F, I> {
    fn _loop_handle(&self) -> reactor::Handle {
        self.inner.with(move |inner: &mut ServerInner<F>| inner.session_state.loop_handle.clone())
    }

    fn process_response_part(self, stream_id: StreamId, part: HttpStreamPart) -> HttpFuture<Self> {
        let stream_id = self.inner.with(move |inner: &mut ServerInner<F>| {
            let stream = inner.common.get_stream_mut(stream_id);
            if let Some(stream) = stream {
                if !stream.common.state.is_closed_local() {
                    stream.common.outgoing.push_back(part.content);
                    if part.last {
                        stream.common.outgoing_end = true;
                    }
                    Some(stream_id)
                } else {
                    None
                }
            } else {
                None
            }
        });
        if let Some(stream_id) = stream_id {
            self.send_outg_stream(stream_id)
        } else {
            Box::new(futures::finished(self))
        }
    }

    fn process_response_end(self, stream_id: StreamId) -> HttpFuture<Self> {
        let stream_id = self.inner.with(move |inner: &mut ServerInner<F>| {
            let stream = inner.common.get_stream_mut(stream_id);
            if let Some(stream) = stream {
                stream.common.outgoing_end = true;
                Some(stream_id)
            } else {
                None
            }
        });
        if let Some(stream_id) = stream_id {
            self.send_outg_stream(stream_id)
        } else {
            Box::new(futures::finished(self))
        }
    }

    fn process_message(self, message: ServerToWriteMessage<F>) -> HttpFuture<Self> {
        match message {
            ServerToWriteMessage::ResponsePart(stream_id, response) => self.process_response_part(stream_id, response),
            ServerToWriteMessage::ResponseStreamEnd(stream_id) => self.process_response_end(stream_id),
            ServerToWriteMessage::Common(common) => self.process_common(common),
            ServerToWriteMessage::_Dummy(..) => panic!(),
        }
    }

    fn run(self, requests: HttpFutureStream<ServerToWriteMessage<F>>) -> HttpFuture<()> {
        let requests = requests.map_err(HttpError::from);
        Box::new(requests
            .fold(self, move |wl, message: ServerToWriteMessage<F>| {
                wl.process_message(message)
            })
            .map(|_| ()))
    }
}

impl<F : HttpService> ServerCommandLoop<F> {
    fn process_dump_state(self, sender: futures::sync::oneshot::Sender<ConnectionStateSnapshot>) -> HttpFuture<Self> {
        // ignore send error, client might be already dead
        drop(sender.complete(self.inner.with(|inner| inner.common.dump_state())));
        Box::new(futures::finished(self))
    }

    fn process_message(self, message: ServerCommandMessage) -> HttpFuture<Self> {
        match message {
            ServerCommandMessage::DumpState(sender) => self.process_dump_state(sender),
        }
    }

    fn run(self, requests: HttpFutureStreamSend<ServerCommandMessage>) -> HttpFuture<()> {
        let requests = requests.map_err(HttpError::from);
        Box::new(requests
            .fold(self, move |l, message: ServerCommandMessage| {
                l.process_message(message)
            })
            .map(|_| ()))
    }}



pub struct HttpServerConnectionAsync {
    command_tx: futures::sync::mpsc::UnboundedSender<ServerCommandMessage>,
}

impl HttpServerConnectionAsync {
    fn connected<F, I>(lh: &reactor::Handle, socket: HttpFutureSend<I>, _conf: HttpServerConf, service: Arc<F>)
                       -> (HttpServerConnectionAsync, HttpFuture<()>)
        where
            F : HttpService,
            I : Io + Send + 'static,
    {
        let lh = lh.clone();

        let (to_write_tx, to_write_rx) = futures::sync::mpsc::unbounded::<ServerToWriteMessage<F>>();
        let (command_tx, command_rx) = futures::sync::mpsc::unbounded::<ServerCommandMessage>();

        let to_write_rx = to_write_rx.map_err(|()| HttpError::IoError(io::Error::new(io::ErrorKind::Other, "to_write")));
        let command_rx = Box::new(command_rx.map_err(|()| HttpError::IoError(io::Error::new(io::ErrorKind::Other, "command"))));

        let handshake = socket.and_then(server_handshake);

        let run = handshake.and_then(move |socket| {
            let (read, write) = socket.split();

            let inner = TaskRcMut::new(ServerInner {
                common: LoopInnerCommon::new(HttpScheme::Http),
                session_state: HttpServerSessionState {
                    factory: service,
                    to_write_tx: to_write_tx.clone(),
                    loop_handle: lh,
                },
            });

            let run_write = ServerWriteLoop { write: write, inner: inner.clone() }.run(Box::new(to_write_rx));
            let run_read = ServerReadLoop { read: read, inner: inner.clone() }.run();
            let run_command = ServerCommandLoop { inner: inner.clone() }.run(command_rx);

            run_write.join(run_read).join(run_command).map(|_| ())
        });

        let future = Box::new(run.then(|x| { info!("connection end: {:?}", x); x }));

        (HttpServerConnectionAsync {
            command_tx: command_tx,
        }, future)
    }

    pub fn new_plain<S>(lh: &reactor::Handle, socket: TcpStream, conf: HttpServerConf, service: Arc<S>)
            -> (HttpServerConnectionAsync, HttpFuture<()>)
        where
            S : HttpService,
    {
        HttpServerConnectionAsync::connected(lh, Box::new(futures::finished(socket)), conf, service)
    }

    /*
    pub fn new_tls<F>(lh: &reactor::Handle, socket: TcpStream, server_context: tokio_tls::ServerContext, factory: F)
            -> (HttpServerConnectionAsync, HttpFuture<()>)
        where
            F : HttpService,
    {
        let stream = server_context.handshake(socket).map_err(HttpError::from);

        HttpServerConnectionAsync::connected(lh, Box::new(stream), factory)
    }
    */

    pub fn new_plain_fn<F>(lh: &reactor::Handle, socket: TcpStream, conf: HttpServerConf, f: F)
            -> (HttpServerConnectionAsync, HttpFuture<()>)
        where
            F : Fn(Vec<Header>, HttpPartFutureStreamSend) -> HttpPartFutureStreamSend + Send + 'static,
    {
        struct HttpServiceFn<F>(F);

        impl<F> HttpService for HttpServiceFn<F>
            where F : Fn(Vec<Header>, HttpPartFutureStreamSend) -> HttpPartFutureStreamSend + Send + 'static
        {
            fn new_request(&self, headers: Vec<Header>, req: HttpPartFutureStreamSend) -> HttpPartFutureStreamSend {
                (self.0)(headers, req)
            }
        }

        HttpServerConnectionAsync::new_plain(lh, socket, conf, Arc::new(HttpServiceFn(f)))
    }

    /*
    pub fn new_tls_fn<F>(lh: &reactor::Handle, socket: TcpStream, server_context: tokio_tls::ServerContext, f: F)
            -> (HttpServerConnectionAsync, HttpFuture<()>)
        where
            F : Fn(Vec<Header>, HttpStreamStreamSend) -> HttpStreamStreamSend + Send + 'static,
    {
        struct HttpServiceFn<F>(F);

        impl<F> HttpService for HttpServiceFn<F>
            where F : Fn(Vec<Header>, HttpStreamStreamSend) -> HttpStreamStreamSend + Send + 'static
        {
            fn new_request(&mut self, headers: Vec<Header>, req: HttpStreamStreamSend) -> HttpStreamStreamSend {
                (self.0)(headers, req)
            }
        }

        HttpServerConnectionAsync::new_tls(lh, socket, server_context, HttpServiceFn(f))
    }
    */

    /// For tests
    pub fn dump_state(&self) -> HttpFutureSend<ConnectionStateSnapshot> {
        let (tx, rx) = futures::oneshot();

        self.command_tx.clone().send(ServerCommandMessage::DumpState(tx))
            .expect("send request to dump state");

        let rx = rx.map_err(|_| HttpError::from(io::Error::new(io::ErrorKind::Other, "oneshot canceled")));

        Box::new(rx)
    }

}
