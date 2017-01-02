use std::marker;
use std::io;
use std::sync::Arc;

use solicit::http::frame::*;
use solicit::http::StreamId;
use solicit::http::HttpScheme;
use solicit::http::HttpError;
use solicit::http::Header;
use solicit::http::StaticHeader;

use futures;
use futures::Future;
use futures::stream::Stream;

use tokio_core::io::Io;
use tokio_core::net::TcpStream;
use tokio_core::reactor;

//use tokio_tls;

use futures_misc::*;

use solicit_async::*;
use solicit_misc::*;
use http_common::*;


struct GrpcHttpServerStream<F : HttpService> {
    common: GrpcHttpStreamCommon,
    request_handler: Option<futures::sync::mpsc::UnboundedSender<ResultOrEof<HttpStreamPart, HttpError>>>,
    _marker: marker::PhantomData<F>,
}

impl<F : HttpService> GrpcHttpStream for GrpcHttpServerStream<F> {
    fn common(&self) -> &GrpcHttpStreamCommon {
        &self.common
    }

    fn common_mut(&mut self) -> &mut GrpcHttpStreamCommon {
        &mut self.common
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

    fn closed_remote(&mut self) {
        if let Some(mut sender) = self.request_handler.take() {
            // ignore error
            sender.send(ResultOrEof::Eof).ok();
        }
    }
}

impl<F : HttpService> GrpcHttpServerStream<F> {
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

}

struct GrpcHttpServerSessionState<F : HttpService> {
    factory: Arc<F>,
    to_write_tx: futures::sync::mpsc::UnboundedSender<ServerToWriteMessage<F>>,
    loop_handle: reactor::Handle,
}


struct ServerInner<F : HttpService> {
    common: LoopInnerCommon<GrpcHttpServerStream<F>>,
    session_state: GrpcHttpServerSessionState<F>,
}

impl<F : HttpService> ServerInner<F> {
    fn new_request(&mut self, stream_id: StreamId, headers: Vec<StaticHeader>)
        -> futures::sync::mpsc::UnboundedSender<ResultOrEof<HttpStreamPart, HttpError>>
    {
        let (req_tx, req_rx) = futures::sync::mpsc::unbounded();

        let req_rx = req_rx.map_err(|()| HttpError::from(io::Error::new(io::ErrorKind::Other, "req")));
        let req_rx = stream_with_eof_and_error(req_rx, || HttpError::from(io::Error::new(io::ErrorKind::Other, "unexpected eof")));

        let response = self.session_state.factory.new_request(headers, Box::new(req_rx));

        {
            let mut to_write_tx = self.session_state.to_write_tx.clone();
            let mut to_write_tx2 = to_write_tx.clone();

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

    fn new_stream(&mut self, stream_id: StreamId, headers: Vec<StaticHeader>) -> &mut GrpcHttpServerStream<F> {
        debug!("new stream: {}", stream_id);

        let req_tx = self.new_request(stream_id, headers);

        // New stream initiated by the client
        let stream = GrpcHttpServerStream {
            common: GrpcHttpStreamCommon::new(),
            request_handler: Some(req_tx),
            _marker: marker::PhantomData,
        };
        if let Some(..) = self.common.streams.insert(stream_id, stream) {
            panic!("inserted stream that already existed");
        }
        self.common.streams.get_mut(&stream_id).unwrap()
    }

    fn get_or_create_stream(&mut self, stream_id: StreamId, headers: Vec<StaticHeader>, last: bool) -> &mut GrpcHttpServerStream<F> {
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
    type LoopHttpStream = GrpcHttpServerStream<F>;

    fn common(&mut self) -> &mut LoopInnerCommon<GrpcHttpServerStream<F>> {
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
        let headers = headers.into_iter().map(|h| h.into()).collect();

        let _stream = self.get_or_create_stream(
            frame.get_stream_id(),
            headers,
            frame.is_end_of_stream());

        // TODO: drop stream if closed on both ends
    }

}

type ServerReadLoop<F, I> = ReadLoopData<I, ServerInner<F>>;
type ServerWriteLoop<F, I> = WriteLoopData<I, ServerInner<F>>;


enum ServerToWriteMessage<F : HttpService> {
    _Dummy(F),
    ResponsePart(StreamId, HttpStreamPart),
    ResponseStreamEnd(StreamId),
    Common(CommonToWriteMessage),
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
        let send_buf = self.inner.with(move |inner: &mut ServerInner<F>| {
            let (send, close) = {
                let stream = inner.common.get_stream_mut(stream_id);
                if let Some(stream) = stream {
                    if !stream.common.state.is_closed_local() {
                        stream.common.close_local();
                        (true, true)
                    } else {
                        (false, false)
                    }
                } else {
                    (false, false)
                }
            };

            if close {
                inner.common.remove_stream_if_closed(stream_id);
            }

            if send {
                inner.common.conn.send_end_of_stream_to_vec(stream_id).unwrap()
            } else {
                Vec::new()
            }
        });
        self.write_all(send_buf)
    }

    fn process_message(self, message: ServerToWriteMessage<F>) -> HttpFuture<Self> {
        match message {
            ServerToWriteMessage::ResponsePart(stream_id, response) => self.process_response_part(stream_id, response),
            ServerToWriteMessage::ResponseStreamEnd(stream_id) => self.process_response_end(stream_id),
            ServerToWriteMessage::Common(common) => self.process_common(common),
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
    fn connected<F, I>(lh: &reactor::Handle, socket: HttpFutureSend<I>, service: Arc<F>) -> HttpFuture<()>
        where
            F : HttpService,
            I : Io + Send + 'static,
    {
        let lh = lh.clone();

        let (to_write_tx, to_write_rx) = futures::sync::mpsc::unbounded::<ServerToWriteMessage<F>>();

        let handshake = socket.and_then(server_handshake);

        let run = handshake.and_then(move |socket| {
            let (read, write) = socket.split();

            let inner = TaskRcMut::new(ServerInner {
                common: LoopInnerCommon::new(HttpScheme::Http),
                session_state: GrpcHttpServerSessionState {
                    factory: service,
                    to_write_tx: to_write_tx.clone(),
                    loop_handle: lh,
                },
            });

            let to_write_rx = to_write_rx.map_err(|()| HttpError::IoError(io::Error::new(io::ErrorKind::Other, "to_write")));

            let run_write = ServerWriteLoop { write: write, inner: inner.clone() }.run(Box::new(to_write_rx));
            let run_read = ServerReadLoop { read: read, inner: inner.clone() }.run();

            run_write.join(run_read).map(|_| ())
        });

        Box::new(run.then(|x| { info!("connection end: {:?}", x); x }))
    }

    pub fn new_plain<S>(lh: &reactor::Handle, socket: TcpStream, service: Arc<S>) -> HttpFuture<()>
        where
            S : HttpService,
    {
        HttpServerConnectionAsync::connected(lh, Box::new(futures::finished(socket)), service)
    }

    /*
    pub fn new_tls<F>(lh: &reactor::Handle, socket: TcpStream, server_context: tokio_tls::ServerContext, factory: F) -> HttpFuture<()>
        where
            F : HttpService,
    {
        let stream = server_context.handshake(socket).map_err(HttpError::from);

        HttpServerConnectionAsync::connected(lh, Box::new(stream), factory)
    }
    */

    pub fn new_plain_fn<F>(lh: &reactor::Handle, socket: TcpStream, f: F) -> HttpFuture<()>
        where
            F : Fn(Vec<StaticHeader>, HttpStreamStreamSend) -> HttpStreamStreamSend + Send + 'static,
    {
        struct HttpServiceFn<F>(F);

        impl<F> HttpService for HttpServiceFn<F>
            where F : Fn(Vec<StaticHeader>, HttpStreamStreamSend) -> HttpStreamStreamSend + Send + 'static
        {
            fn new_request(&self, headers: Vec<StaticHeader>, req: HttpStreamStreamSend) -> HttpStreamStreamSend {
                (self.0)(headers, req)
            }
        }

        HttpServerConnectionAsync::new_plain(lh, socket, Arc::new(HttpServiceFn(f)))
    }

    /*
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
    */
}
