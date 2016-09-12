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
use futures::stream::Stream;
use tokio_core;
use tokio_core::reactor;
use tokio_core::net::TcpListener;

use method::*;
use error::*;
use futures_grpc::*;
use futures_misc::*;
use solicit_async::*;
use solicit_misc::*;
use grpc::*;
use assert_types::*;
use http_server::*;
use http_common::*;


pub trait MethodHandler<Req, Resp> {
    fn handle(&self, req: Req) -> GrpcStreamSend<Resp>;
}

pub struct MethodHandlerUnary<F> {
    f: F
}

pub struct MethodHandlerServerStreaming<F> {
    f: F
}

pub struct MethodHandlerClientStreaming<F> {
    _f: F
}

pub struct MethodHandlerBidi<F> {
    _f: F
}

impl<F> MethodHandlerUnary<F> {
    pub fn new<Req, Resp>(f: F) -> Self
        where F : Fn(Req) -> GrpcFutureSend<Resp>
    {
        MethodHandlerUnary {
            f: f,
        }
    }
}

impl<F> MethodHandlerClientStreaming<F> {
    pub fn new<Req, Resp>(f: F) -> Self
        where F : Fn(GrpcStreamSend<Req>) -> GrpcFutureSend<Resp>
    {
        MethodHandlerClientStreaming {
            _f: f,
        }
    }
}

impl<F> MethodHandlerServerStreaming<F> {
    pub fn new<Req, Resp>(f: F) -> Self
        where F : Fn(Req) -> GrpcStreamSend<Resp>
    {
        MethodHandlerServerStreaming {
            f: f,
        }
    }
}

impl<F> MethodHandlerBidi<F> {
    pub fn new<Req, Resp>(f: F) -> Self
        where F : Fn(GrpcStreamSend<Req>) -> GrpcStreamSend<Resp>
    {
        MethodHandlerBidi {
            _f: f,
        }
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerUnary<F>
    where
        Resp : Send + 'static,
        F : Fn(Req) -> GrpcFutureSend<Resp>,
{
    fn handle(&self, req: Req) -> GrpcStreamSend<Resp> {
        Box::new(future_to_stream_once((self.f)(req)))
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerClientStreaming<F>
    where
        Resp : Send + 'static,
        F : Fn(GrpcStreamSend<Req>) -> GrpcFutureSend<Resp>,
{
    fn handle(&self, _req: Req) -> GrpcStreamSend<Resp> {
        unimplemented!()
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerServerStreaming<F>
    where
        Resp : Send + 'static,
        F : Fn(Req) -> GrpcStreamSend<Resp>,
{
    fn handle(&self, req: Req) -> GrpcStreamSend<Resp> {
        Box::new((self.f)(req))
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerBidi<F>
    where
        Resp : Send + 'static,
        F : Fn(GrpcStreamSend<Req>) -> GrpcStreamSend<Resp>,
{
    fn handle(&self, _req: Req) -> GrpcStreamSend<Resp> {
        unimplemented!()
    }
}


trait MethodHandlerDispatch {
    fn on_message(&self, message: &[u8]) -> GrpcStreamSend<Vec<u8>>;
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
    fn on_message(&self, message: &[u8]) -> GrpcStreamSend<Vec<u8>> {
        let req = match self.desc.req_marshaller.read(message) {
            Ok(req) => req,
            Err(e) => return Box::new(future_to_stream_once(futures::done(Err(e)))),
        };
        let resp =
            catch_unwind(AssertUnwindSafe(|| self.method_handler.handle(req)));
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
    pub fn handle_method(&self, name: &str, message: &[u8]) -> GrpcStreamSend<Vec<u8>> {
        self.find_method(name).dispatch.on_message(message)
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

struct ProcessRequestShared {
    service_definition: Arc<ServerServiceDefinition>,
    resp_tx: tokio_core::channel::Sender<ResultOrEof<HttpStreamPart, HttpError>>,
    remote: reactor::Remote,
}

enum ProcessRequestState {
    Initial(ProcessRequestShared),
    Body(ProcessRequestShared, String, Vec<u8>),
}

impl ProcessRequestState {
    fn initial(shared: ProcessRequestShared, message: HttpStreamPart) -> HttpFutureSend<ProcessRequestState> {
        let headers = match message.content {
            HttpStreamPartContent::Headers(headers) => headers,
            HttpStreamPartContent::Data(_data) => panic!("data before headers"), // TODO: send some response
        };

        let path = slice_get_header(&headers, ":path").expect(":path header").to_owned();

        Box::new(futures::finished(ProcessRequestState::Body(shared, path, Vec::new())))
    }

    fn body(shared: ProcessRequestShared, path: String, mut body: Vec<u8>, message: HttpStreamPart) -> HttpFutureSend<ProcessRequestState> {
        match message.content {
            // that's unexpected
            HttpStreamPartContent::Headers(_header) => Box::new(futures::finished(ProcessRequestState::Body(shared, path, body))),
            HttpStreamPartContent::Data(data) => {
                body.extend(&data);
                Box::new(futures::finished(ProcessRequestState::Body(shared, path, body)))
            }
        }
    }

    fn message(self, message: HttpStreamPart) -> HttpFutureSend<ProcessRequestState> {
        match self {
            ProcessRequestState::Initial(shared) => ProcessRequestState::initial(shared, message),
            ProcessRequestState::Body(shared, path, body) => ProcessRequestState::body(shared, path, body, message),
        }
    }

    fn finish(self) -> HttpFutureSend<()> {
        let (shared, path, data) = match self {
            ProcessRequestState::Initial(..) => panic!("no headers yet"),
            ProcessRequestState::Body(shared, path, data) => (shared, path, data),
        };

        let message = parse_grpc_frame_completely(&data).expect("parse req body");

        let stream = shared.service_definition.handle_method(&path, message);

        enum ResponseProcessState {
            None,
            HeadersAndSomeDataSent,
        }

        let shared: ProcessRequestShared = shared;
        let resp_tx_1 = shared.resp_tx.clone();
        let resp_tx_2 = shared.resp_tx.clone();

        let future = stream.fold(ResponseProcessState::None, move |state, response| {
            if let ResponseProcessState::None = state {
                resp_tx_1.send(ResultOrEof::Item(HttpStreamPart::intermediate_headers(
                    vec![
                        Header::new(":status", "200"),
                    ]),
                )).unwrap();
            }

            let http_message = write_grpc_frame_to_vec(&response);

            resp_tx_1.send(ResultOrEof::Item(HttpStreamPart::intermediate_data(http_message))).unwrap();

            futures::finished::<_, GrpcError>(ResponseProcessState::HeadersAndSomeDataSent)
        }).then(move |r| {
            match r {
                Ok(state) => {
                    match state {
                        ResponseProcessState::None => {
                            resp_tx_2.send(ResultOrEof::Item(HttpStreamPart::last_headers(vec![
                                Header::new(":status", "500"),
                                Header::new(HEADER_GRPC_MESSAGE, "empty response"),
                            ]))).unwrap();
                        },
                        ResponseProcessState::HeadersAndSomeDataSent => {
                            resp_tx_2.send(ResultOrEof::Item(HttpStreamPart::last_headers(vec![
                                Header::new(HEADER_GRPC_STATUS, "0"),
                            ]))).unwrap();
                        }
                    }
                },
                Err(e) => {
                    resp_tx_2.send(ResultOrEof::Item(HttpStreamPart::last_headers(vec![
                        Header::new(":status", "500"),
                        Header::new(HEADER_GRPC_MESSAGE, format!("error: {:?}", e)),
                    ]))).unwrap();
                },
            }

            resp_tx_2.send(ResultOrEof::Eof).unwrap();

            Ok(())
        });

        shared.remote.spawn(move |_handle| future);

        Box::new(futures::finished(()))
    }
}

impl HttpServerHandlerFactory for GrpcHttpServerHandlerFactory {
    fn new_request(&mut self, handle: &reactor::Handle, req: HttpStreamStreamSend) -> HttpStreamStreamSend {
        let (resp_tx, resp_rx) = tokio_core::channel::channel(handle).unwrap();

        let shared = ProcessRequestShared {
            service_definition: self.service_definition.clone(),
            resp_tx: resp_tx,
            remote: handle.remote().clone(),
        };
        let request_future = req
            .fold(ProcessRequestState::Initial(shared), ProcessRequestState::message)
            .and_then(ProcessRequestState::finish)
            .map_err(|e| { panic!("{:?}", e); });
        handle.spawn(request_future);

        let resp_rx = resp_rx.map_err(HttpError::from);

        Box::new(stream_with_eof_and_error(resp_rx))
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
