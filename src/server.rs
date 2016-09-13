use std::mem;
use std::sync::Arc;
use std::thread;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::sync::mpsc;
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;
use std::any::Any;

use solicit::http::HttpError;
use solicit::http::Header;

use futures;
use futures::Future;
use futures::Poll;
use futures::Async;
use futures::stream::Stream;
use tokio_core;
use tokio_core::reactor;
use tokio_core::net::TcpListener;

use method::*;
use error::*;
use futures_grpc::*;
use futures_misc::*;
use solicit_misc::*;
use grpc::*;
use assert_types::*;
use http_server::*;
use http_common::*;


pub trait MethodHandler<Req, Resp> {
    fn handle(&self, req: GrpcStreamSend<Req>) -> GrpcStreamSend<Resp>;
}

pub struct MethodHandlerUnary<F> {
    f: Arc<F>
}

pub struct MethodHandlerServerStreaming<F> {
    f: Arc<F>
}

pub struct MethodHandlerClientStreaming<F> {
    _f: Arc<F>
}

pub struct MethodHandlerBidi<F> {
    _f: Arc<F>
}

impl<F> MethodHandlerUnary<F> {
    pub fn new<Req, Resp>(f: F) -> Self
        where F : Fn(Req) -> GrpcFutureSend<Resp> + Send + Sync,
    {
        MethodHandlerUnary {
            f: Arc::new(f),
        }
    }
}

impl<F> MethodHandlerClientStreaming<F> {
    pub fn new<Req, Resp>(f: F) -> Self
        where F : Fn(GrpcStreamSend<Req>) -> GrpcFutureSend<Resp> + Send + Sync,
    {
        MethodHandlerClientStreaming {
            _f: Arc::new(f),
        }
    }
}

impl<F> MethodHandlerServerStreaming<F> {
    pub fn new<Req, Resp>(f: F) -> Self
        where F : Fn(Req) -> GrpcStreamSend<Resp> + Send + Sync,
    {
        MethodHandlerServerStreaming {
            f: Arc::new(f),
        }
    }
}

impl<F> MethodHandlerBidi<F> {
    pub fn new<Req, Resp>(f: F) -> Self
        where F : Fn(GrpcStreamSend<Req>) -> GrpcStreamSend<Resp> + Send + Sync,
    {
        MethodHandlerBidi {
            _f: Arc::new(f),
        }
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerUnary<F>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
        F : Fn(Req) -> GrpcFutureSend<Resp> + Send + Sync + 'static,
{
    fn handle(&self, req: GrpcStreamSend<Req>) -> GrpcStreamSend<Resp> {
        let f = self.f.clone();
        Box::new(future_to_stream_once(stream_single_send(req).and_then(move |req| f(req))))
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerClientStreaming<F>
    where
        Resp : Send + 'static,
        F : Fn(GrpcStreamSend<Req>) -> GrpcFutureSend<Resp> + Send + Sync + 'static,
{
    fn handle(&self, _req: GrpcStreamSend<Req>) -> GrpcStreamSend<Resp> {
        unimplemented!()
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerServerStreaming<F>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
        F : Fn(Req) -> GrpcStreamSend<Resp> + Send + Sync + 'static,
{
    fn handle(&self, req: GrpcStreamSend<Req>) -> GrpcStreamSend<Resp> {
        let f = self.f.clone();
        Box::new(
            future_flatten_to_stream(
                stream_single_send(req).map(move |req| f(req))))
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerBidi<F>
    where
        Resp : Send + 'static,
        F : Fn(GrpcStreamSend<Req>) -> GrpcStreamSend<Resp> + Send + Sync + 'static,
{
    fn handle(&self, _req: GrpcStreamSend<Req>) -> GrpcStreamSend<Resp> {
        unimplemented!()
    }
}


trait MethodHandlerDispatch {
    fn start_request(&self, req: GrpcStreamSend<Vec<u8>>) -> GrpcStreamSend<Vec<u8>>;
}

struct MethodHandlerDispatchAsyncImpl<Req, Resp> {
    desc: Arc<MethodDescriptor<Req, Resp>>,
    method_handler: Box<MethodHandler<Req, Resp> + Sync + Send>,
}

fn any_to_string(any: Box<Any + Send + 'static>) -> String {
    if any.is::<String>() {
        *any.downcast::<String>().unwrap()
    } else if any.is::<&str>() {
        (*any.downcast::<&str>().unwrap()).to_owned()
    } else {
        "unknown any".to_owned()
    }
}

impl<Req, Resp> MethodHandlerDispatch for MethodHandlerDispatchAsyncImpl<Req, Resp>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
{
    fn start_request(&self, req_grpc_frames: GrpcStreamSend<Vec<u8>>) -> GrpcStreamSend<Vec<u8>> {
        let desc = self.desc.clone();
        let req = req_grpc_frames.and_then(move |frame| desc.req_marshaller.read(&frame));
        let resp =
            catch_unwind(AssertUnwindSafe(|| self.method_handler.handle(Box::new(req))));
        match resp {
            Ok(resp) => {
                let desc_copy = self.desc.clone();
                Box::new(resp
                    .and_then(move |resp| desc_copy.resp_marshaller.write(&resp)))
            }
            Err(e) => {
                let message = any_to_string(e);
                Box::new(future_to_stream_once(futures::failed(GrpcError::Panic(message))))
            }
        }
    }
}

pub struct ServerMethod {
    name: String,
    dispatch: Box<MethodHandlerDispatch + Sync + Send>,
}

impl ServerMethod {
    pub fn new<Req, Resp, H>(method: Arc<MethodDescriptor<Req, Resp>>, handler: H) -> ServerMethod
        where
            Req : Send + 'static,
            Resp : Send + 'static,
            H : MethodHandler<Req, Resp> + 'static + Sync + Send,
    {
        ServerMethod {
            name: method.name.clone(),
            dispatch: Box::new(MethodHandlerDispatchAsyncImpl {
                desc: method,
                method_handler: Box::new(handler),
            }),
        }
    }
}

pub struct ServerServiceDefinition {
    methods: Vec<ServerMethod>,
}

impl ServerServiceDefinition {
    pub fn new(methods: Vec<ServerMethod>) -> ServerServiceDefinition {
        ServerServiceDefinition {
            methods: methods,
        }
    }

    pub fn find_method(&self, name: &str) -> &ServerMethod {
        self.methods.iter()
            .filter(|m| m.name == name)
            .next()
            .expect(&format!("unknown method: {}", name))
    }

    // TODO: stream
    pub fn handle_method(&self, name: &str, message: GrpcStreamSend<Vec<u8>>) -> GrpcStreamSend<Vec<u8>> {
        self.find_method(name).dispatch.start_request(message)
    }
}


struct LoopToServer {
    // used only once to send shutdown signal
    shutdown_tx: tokio_core::channel::Sender<()>,
    local_addr: SocketAddr,
}

fn _assert_loop_to_server() {
    assert_send::<LoopToServer>();
}


pub struct GrpcServer {
    loop_to_server: LoopToServer,
    thread_join_handle: Option<thread::JoinHandle<()>>,
}

impl GrpcServer {
    pub fn new(port: u16, service_definition: ServerServiceDefinition) -> GrpcServer {
        let listen_addr = ("::", port).to_socket_addrs().unwrap().next().unwrap();

        let (get_from_loop_tx, get_from_loop_rx) = mpsc::channel();

        let join_handle = thread::spawn(move || {
            run_server_event_loop(listen_addr, service_definition, get_from_loop_tx);
        });

        let loop_to_server = get_from_loop_rx.recv().unwrap();

        GrpcServer {
            loop_to_server: loop_to_server,
            thread_join_handle: Some(join_handle),
        }
    }

    pub fn local_addr(&self) -> &SocketAddr {
        &self.loop_to_server.local_addr
    }
}

// We shutdown client in destructor.
impl Drop for GrpcServer {
    fn drop(&mut self) {
        // ignore error because even loop may be already dead
        self.loop_to_server.shutdown_tx.send(()).ok();

        // do not ignore errors because we own event loop thread
        self.thread_join_handle.take().unwrap().join().unwrap();
    }
}

struct GrpcHttpServerHandlerFactory {
    service_definition: Arc<ServerServiceDefinition>,
}


struct GrpcFrameFromHttpFramesStream {
    http_stream_stream: HttpStreamStreamSend,
    buf: Vec<u8>,
}

impl Stream for GrpcFrameFromHttpFramesStream {
    type Item = Vec<u8>;
    type Error = GrpcError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if let Some((frame, len)) = try!(parse_grpc_frame(&self.buf)).map(|(frame, len)| (frame.to_owned(), len)) {
                self.buf.drain(..len);
                return Ok(Async::Ready(Some(frame)));
            }

            let part_opt = try_ready!(self.http_stream_stream.poll());
            let part = match part_opt {
                None => {
                    if self.buf.is_empty() {
                        return Ok(Async::Ready(None));
                    } else {
                        return Err(GrpcError::Other("partial frame"));
                    }
                },
                Some(part) => part,
            };

            match part.content {
                // unexpected, but seem OK
                HttpStreamPartContent::Headers(..) => continue,
                HttpStreamPartContent::Data(data) => self.buf.extend(data),
            }
        }
    }
}


enum ProcessRequestState {
    Request(HttpStreamStreamSend),
    Response(HttpStreamStreamSend),
    Stub,
}

struct ProcessRequestStream {
    service_definition: Arc<ServerServiceDefinition>,
    state: ProcessRequestState,
}

impl ProcessRequestStream {
    fn poll_send_500(&mut self, message: &str) -> Poll<Option<HttpStreamPart>, HttpError> {
        Ok(Async::Ready(Some(HttpStreamPart::last_headers(vec![
            Header::new(":status", "500"),
            Header::new(HEADER_GRPC_MESSAGE, message.to_owned()),
        ]))))
    }

    fn poll_part_initial(&mut self, part: HttpStreamPart) -> Poll<Option<HttpStreamPart>, HttpError> {
        let headers = match part.content {
            HttpStreamPartContent::Data(..) => return self.poll_send_500("body before headers"),
            HttpStreamPartContent::Headers(headers) => headers,
        };

        let path = match slice_get_header(&headers, ":path") {
            Some(path) => path.to_owned(),
            None => return self.poll_send_500("no :path header"),
        };

        let http_request = match mem::replace(&mut self.state, ProcessRequestState::Stub) {
            ProcessRequestState::Request(http_request) => http_request,
            _ => unreachable!(),
        };

        let grpc_request = GrpcFrameFromHttpFramesStream {
            http_stream_stream: http_request,
            buf: Vec::new(),
        };

        // TODO: catch unwind
        let grpc_frames = self.service_definition.handle_method(&path, Box::new(grpc_request));

        let http_parts = stream_concat(
            grpc_frames
                .map(|frame| HttpStreamPart::intermediate_data(write_grpc_frame_to_vec(&frame)))
                .then(|result| {
                    match result {
                        Ok(part) => { let r: Result<_, HttpError> = Ok(part); r }
                        Err(e) =>
                            Ok(HttpStreamPart::last_headers(vec![
                                Header::new(":status", "500"),
                                Header::new(HEADER_GRPC_MESSAGE, format!("error: {:?}", e)),
                            ]))
                    }
                }),
                stream_once_send(HttpStreamPart::last_headers(vec![
                    Header::new(HEADER_GRPC_STATUS, "0"),
                ])));

        self.state = ProcessRequestState::Response(Box::new(http_parts));
        Ok(Async::NotReady)
    }

    fn poll_part(&mut self, part: HttpStreamPart) -> Poll<Option<HttpStreamPart>, HttpError> {
        match self.state {
            ProcessRequestState::Request(..) => self.poll_part_initial(part),
            _ => unreachable!(),
        }
    }

    fn poll_eof(&mut self) -> Poll<Option<HttpStreamPart>, HttpError> {
        Ok(Async::Ready(Some(HttpStreamPart::last_headers(vec![
            Header::new(":status", "500"),
            Header::new(HEADER_GRPC_MESSAGE, "no header"),
        ]))))
    }

    fn poll_without_repoll(&mut self, part: Option<HttpStreamPart>) -> Poll<Option<HttpStreamPart>, HttpError> {
        match part {
            Some(part) => self.poll_part(part),
            None => self.poll_eof(),
        }
    }

    fn poll_with_repoll(&mut self) -> Poll<Option<HttpStreamPart>, HttpError> {
        loop {
            let part = match self.state {
                ProcessRequestState::Request(ref mut request) => {
                    try_ready!(request.poll())
                },
                ProcessRequestState::Response(ref mut s) => return s.poll(),
                ProcessRequestState::Stub => unreachable!(),
            };

            match self.poll_without_repoll(part) {
                Err(e) => return Err(e),
                Ok(Async::Ready(part)) => return Ok(Async::Ready(part)),
                Ok(Async::NotReady) => (), // repoll
            };
        }
    }
}

impl Stream for ProcessRequestStream {
    type Item = HttpStreamPart;
    type Error = HttpError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.poll_with_repoll()
    }
}

impl HttpServerHandlerFactory for GrpcHttpServerHandlerFactory {
    fn new_request(&mut self, _handle: &reactor::Handle, req: HttpStreamStreamSend) -> HttpStreamStreamSend {
        let s = ProcessRequestStream {
            service_definition: self.service_definition.clone(),
            state: ProcessRequestState::Request(req),
        };
        Box::new(s)
    }
}

fn run_server_event_loop(
    listen_addr: SocketAddr,
    service_definition: ServerServiceDefinition,
    send_to_back: mpsc::Sender<LoopToServer>)
{
    let service_definition = Arc::new(service_definition);

    let mut lp = reactor::Core::new().expect("loop new");

    let (shutdown_tx, shutdown_rx) = tokio_core::channel::channel(&lp.handle()).unwrap();

    let shutdown_rx = shutdown_rx.map_err(GrpcError::from);

    let listen = TcpListener::bind(&listen_addr, &lp.handle()).unwrap();

    let stuff = stream_repeat(lp.handle());

    let local_addr = listen.local_addr().unwrap();
    send_to_back
        .send(LoopToServer { shutdown_tx: shutdown_tx, local_addr: local_addr })
        .expect("send back");

    let loop_run = listen.incoming().map_err(GrpcError::from).zip(stuff).for_each(move |((socket, _peer_addr), loop_handle)| {
        loop_handle.spawn(HttpServerConnectionAsync::new(&loop_handle, socket, GrpcHttpServerHandlerFactory {
            service_definition: service_definition.clone(),
        }).map_err(|e| { println!("{:?}", e); () }));
        Ok(())
    });

    let shutdown = shutdown_rx.into_future().map_err(|(e, _)| GrpcError::from(e)).and_then(|_| {
        // Must complete with error,
        // so `join` with this future cancels another future.
        futures::failed::<(), _>(GrpcError::Other("shutdown"))
    });

    // Wait for either completion of connection (i. e. error)
    // or shutdown signal.
    let done = loop_run.join(shutdown);

    // TODO: do not ignore error
    lp.run(done).ok();
}
