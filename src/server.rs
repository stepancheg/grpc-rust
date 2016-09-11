use std::sync::Arc;
use std::thread;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::io;
use std::iter;
use std::sync::mpsc;
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;
use std::any::Any;

use solicit::http::server::StreamFactory;
use solicit::http::server::ServerSession;
use solicit::http::session::Server;
use solicit::http::session::DefaultSessionState;
use solicit::http::session::DefaultStream;
use solicit::http::session::StreamState;
use solicit::http::session::StreamDataChunk;
use solicit::http::session::StreamDataError;
use solicit::http::session::Stream as solicit_Stream;
use solicit::http::connection::HttpConnection;
use solicit::http::connection::EndStream;
use solicit::http::connection::DataChunk;
use solicit::http::connection::HttpFrame;
use solicit::http::HttpScheme;
use solicit::http::StreamId;
use solicit::http::Header;
use solicit::http::session::SessionState;

use futures;
use futures::Future;
use futures::stream;
use futures::stream::Stream;
use tokio_core::io as tokio_io;
use tokio_core::io::ReadHalf;
use tokio_core::io::WriteHalf;
use tokio_core;
use tokio_core::reactor;
use tokio_core::io::Io;
use tokio_core::net::TcpStream;
use tokio_core::net::TcpListener;
use tokio_core::channel::Sender;
use tokio_core::channel::Receiver;

use method::*;
use error::*;
use futures_grpc::*;
use futures_misc::*;
use solicit_async::*;
use solicit_misc::*;
use grpc::*;
use assert_types::*;


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

    pub fn handle_method(&self, name: &str, message: &[u8]) -> GrpcStream<Vec<u8>> {
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

struct GrpcHttp2ServerStream {
    stream: DefaultStream,
    service_definition: Arc<ServerServiceDefinition>,
    path: String,
    sender: Sender<ReadToWriteMessage>,
    loop_handle: reactor::Handle,
}

impl solicit_Stream for GrpcHttp2ServerStream {
    fn set_headers(&mut self, headers: Vec<Header>) {
        for h in &headers {
            if h.name() == b":path" {
                self.path = String::from_utf8(h.value().to_owned()).unwrap();
            }
        }

        self.stream.set_headers(headers)
    }

    fn new_data_chunk(&mut self, data: &[u8]) {
        self.stream.new_data_chunk(data)
    }

    fn set_state(&mut self, state: StreamState) {
        println!("set state {:?}", state);
        if state == StreamState::HalfClosedRemote {
            let stream_id = self.stream.stream_id.unwrap();

            let message = parse_grpc_frame_completely(&self.stream.body).expect("parse req body");

            let stream = self.service_definition.handle_method(&self.path, message);

            let sender = self.sender.clone();
            self.loop_handle.spawn(
                stream
                    .fold((sender.clone(), 0), move |(sender, i), resp| {
                        sender.send(ReadToWriteMessage {
                            stream_id: stream_id,
                            content: ResponseChunk::Resp(resp, i == 0),
                        }).unwrap();
                        futures::done::<_, GrpcError>(Ok((sender, i + 1)))
                    })
                    .then(move |result| {
                        sender.send(ReadToWriteMessage {
                            stream_id: stream_id,
                            content: match result {
                                Ok(..) => ResponseChunk::DoneSuccess,
                                Err(e) => ResponseChunk::Error(e),
                            }
                        }).expect("send to write");

                        futures::done::<_, GrpcError>(Ok(()))
                    })
                    .then(|_| Ok(())) // TODO: should not probably ignore errors
                );
        }
        self.stream.set_state(state)
    }

    fn get_data_chunk(&mut self, buf: &mut [u8]) -> Result<StreamDataChunk, StreamDataError> {
        self.stream.get_data_chunk(buf)
    }

    fn state(&self) -> StreamState {
        self.stream.state()
    }
}

struct GrpcStreamFactory {
    service_definition: Arc<ServerServiceDefinition>,
    sender: Sender<ReadToWriteMessage>,
    loop_handle: reactor::Handle,
}

impl StreamFactory for GrpcStreamFactory {
	type Stream = GrpcHttp2ServerStream;

	fn create(&mut self, stream_id: StreamId) -> GrpcHttp2ServerStream {
        println!("new stream {}", stream_id);
        GrpcHttp2ServerStream {
            stream: DefaultStream::with_id(stream_id),
            service_definition: self.service_definition.clone(),
            path: String::new(),
            sender: self.sender.clone(),
            loop_handle: self.loop_handle.clone(),
        }
	}
}

struct ServerSharedState {
    conn: HttpConnection,
    factory: GrpcStreamFactory,
    state: DefaultSessionState<Server, GrpcHttp2ServerStream>,
}

enum ResponseChunk {
    Resp(/* resp */ Vec<u8>, /* first */ bool),
    DoneSuccess,
    Error(GrpcError),
}

struct ReadToWriteMessage {
    stream_id: StreamId,
    content: ResponseChunk,
}

fn run_read(
    read: ReadHalf<TcpStream>,
    shared: TaskRcMutex<ServerSharedState>)
        -> GrpcFuture<()>
{
    let stream = stream::iter(iter::repeat(()).map(|x| Ok(x)));

    let future = stream.fold((read, shared), move |(read, shared), _| {
        recv_raw_frame(read)
            .map(|(read, raw_frame)| {
                println!("server: received frame");

                shared.with(|shared: &mut ServerSharedState| {

                    let mut send_buf = VecSendFrame(Vec::new());

                    let mut session = ServerSession::new(&mut shared.state, &mut shared.factory, &mut send_buf);
                    shared.conn.handle_frame(HttpFrame::from_raw(&raw_frame).unwrap(), &mut session).unwrap();

                    // TODO: process send
                });
                (read, shared)
            })
    });

    Box::new(future
        .map(|_| ()))
}

fn run_write(
    write: WriteHalf<TcpStream>,
    shared: TaskRcMutex<ServerSharedState>,
    receiver: Receiver<ReadToWriteMessage>)
    -> GrpcFuture<()>
{
    println!("run_write");
    let future = receiver.fold((write, shared), |(write, shared), message: ReadToWriteMessage| {
        println!("writer got message from reader");

        let send_buf = shared.with(move |shared: &mut ServerSharedState| {
            let mut send_buf = VecSendFrame(Vec::new());

            match message.content {
                ResponseChunk::Resp(resp_bytes, first) => {
                    if first {
                        println!("server: sending headers");
                        shared.conn.sender(&mut send_buf).send_headers(
                            vec![
                            Header::new(b":status", b"200"),
                            Header::new(&b"content-type"[..], &b"application/grpc"[..]),
                        ],
                            message.stream_id,
                            EndStream::No).unwrap();
                    }

                    let mut body = Vec::new();
                    write_grpc_frame(&mut body, &resp_bytes);

                    shared.conn.sender(&mut send_buf).send_data(
                        DataChunk::new_borrowed(&body[..], message.stream_id, EndStream::No)).unwrap();
                }
                ResponseChunk::DoneSuccess => {
                    shared.conn.sender(&mut send_buf).send_headers(
                        vec![
                            Header::new(HEADER_GRPC_STATUS.as_bytes(), b"0"),
                        ],
                        message.stream_id,
                        EndStream::Yes)
                            .unwrap();
                }
                ResponseChunk::Error(e) => {
                    shared.conn.sender(&mut send_buf).send_headers(
                        vec![
                            Header::new(&b":status"[..], &b"500"[..]),
                            Header::new(HEADER_GRPC_MESSAGE.as_bytes(), format!("{:?}", e).into_bytes()),
                        ],
                        message.stream_id,
                        EndStream::Yes).unwrap();

                    shared.state.remove_stream(message.stream_id).expect("unknown stream id");
                }
            }

            send_buf.0
        });

        tokio_io::write_all(write, send_buf)
            .map(|(write, _)| (write, shared))
    });

    Box::new(future
        .map(|_| ())
        .map_err(GrpcError::from))
}


fn run_connection(
    socket: TcpStream,
    peer_addr: SocketAddr,
    service_defintion: Arc<ServerServiceDefinition>,
    loop_handle: reactor::Handle)
    -> Box<Future<Item=(), Error=io::Error>>
{
    println!("accepted connection from {}", peer_addr);
    let handshake = server_handshake(socket).map_err(GrpcError::from);

    let loop_handle_for_channel = loop_handle.clone();
    let three = handshake.and_then(move |conn| {
        let (sender, receiver) = tokio_core::channel::channel(&loop_handle_for_channel).unwrap();
        Ok((conn, sender, receiver))
    });

    let run = three.and_then(|(conn, sender, receiver_stream)| {
        let (read, write) = conn.split();

        let shared_for_read = TaskRcMutex::new(ServerSharedState {
            conn: HttpConnection::new(HttpScheme::Http),
            state: DefaultSessionState::<Server, _>::new(),
            factory: GrpcStreamFactory {
                service_definition: service_defintion,
                sender: sender,
                loop_handle: loop_handle,
            },
        });
        let shared_for_write = shared_for_read.clone();

        run_read(read, shared_for_read)
            .join(run_write(write, shared_for_write, receiver_stream))
    });

    let bye = run
        .then(move |x| {
            println!("closing connection from {}: {:?}", peer_addr, x);
            x
        });

    Box::new(bye
        .map(move |_| ())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e)))
}

fn run_server_event_loop(
    listen_addr: SocketAddr,
    service_definition: ServerServiceDefinition,
    send_to_back: mpsc::Sender<LoopToServer>)
{
    let mut lp = reactor::Core::new().expect("loop new");

    let (shutdown_tx, shutdown_rx) = tokio_core::channel::channel(&lp.handle()).unwrap();

    let shutdown_rx = shutdown_rx.map_err(GrpcError::from);

    let listen = TcpListener::bind(&listen_addr, &lp.handle()).unwrap();

    let stuff = stream_repeat((Arc::new(service_definition), lp.handle()));

    let local_addr = listen.local_addr().unwrap();
    send_to_back
        .send(LoopToServer { shutdown_tx: shutdown_tx, local_addr: local_addr })
        .expect("send back");

    let loop_run = listen.incoming().map_err(GrpcError::from).zip(stuff).for_each(|((socket, peer_addr), (service_definition, loop_handle))| {
        let loop_handle_for_conn = loop_handle.clone();
        loop_handle.spawn(run_connection(socket, peer_addr, service_definition, loop_handle_for_conn).map_err(|_| ()));
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
