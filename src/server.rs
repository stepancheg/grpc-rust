use std::sync::Arc;
use std::thread;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::io;
use std::iter;
use std::sync::mpsc;
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;

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
use solicit::http::HttpScheme;
use solicit::http::StreamId;
use solicit::http::Header;
use solicit::http::session::SessionState;

use futures;
use futures::Future;
use futures::stream;
use futures::stream::Stream;
use tokio_core::io as tokio_io;
use tokio_core::io::IoFuture;
use tokio_core::io::TaskIo;
use tokio_core::io::TaskIoRead;
use tokio_core::io::TaskIoWrite;
use tokio_core;
use tokio_core::Loop;
use tokio_core::LoopHandle;
use tokio_core::TcpStream;
use tokio_core::Sender;
use tokio_core::Receiver;

use method::*;
use error::*;
use result::*;
use futures_grpc::*;
use futures_misc::*;
use solicit_async::*;
use solicit_misc::*;
use grpc::*;


pub trait MethodHandler<Req, Resp> {
    fn handle(&self, req: Req) -> GrpcStream<Resp>;
}

pub struct MethodHandlerFn<F> {
    f: F
}

impl<F> MethodHandlerFn<F> {
    pub fn new<Req, Resp>(f: F)
        -> Self
        where F : Fn(Req) -> GrpcFuture<Resp>
    {
        MethodHandlerFn {
            f: f,
        }
    }
}

impl<Req, Resp, F> MethodHandler<Req, Resp> for MethodHandlerFn<F>
    where
        Resp : Send + 'static,
        F : Fn(Req) -> GrpcFuture<Resp>,
{
    fn handle(&self, req: Req) -> GrpcStream<Resp> {
        future_to_stream((self.f)(req))
            .boxed()
    }
}

trait MethodHandlerDispatch {
    fn on_message(&self, message: &[u8]) -> GrpcStream<Vec<u8>>;
}

struct MethodHandlerDispatchAsyncImpl<Req, Resp> {
    desc: Arc<MethodDescriptor<Req, Resp>>,
    method_handler: Box<MethodHandler<Req, Resp> + Sync + Send>,
}

impl<Req, Resp> MethodHandlerDispatch for MethodHandlerDispatchAsyncImpl<Req, Resp>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
{
    fn on_message(&self, message: &[u8]) -> GrpcStream<Vec<u8>> {
        let req = match self.desc.req_marshaller.read(message) {
            Ok(req) => req,
            Err(e) => return future_to_stream(futures::done(Err(e))).boxed(),
        };
        let resp =
            catch_unwind(AssertUnwindSafe(|| self.method_handler.handle(req)));
        match resp {
            Ok(resp) => {
                let desc_copy = self.desc.clone();
                resp
                    .and_then(move |resp| desc_copy.resp_marshaller.write(&resp))
                    .boxed()
            }
            Err(..) => future_to_stream(futures::done(Err(GrpcError::Other("panic in handler"))))
                .boxed(),
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
    shutdown_tx: tokio_core::Sender<()>,
    local_addr: SocketAddr,
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

            let message = parse_grpc_frame_completely(&self.stream.body).unwrap();

            let sender = self.sender.clone();
            self.service_definition.handle_method(&self.path, message)
                .fold(sender.clone(), move |sender, result| {
                    sender.send(ReadToWriteMessage {
                        stream_id: stream_id,
                        content: Ok(result),
                    }).unwrap();
                    futures::done::<_, GrpcError>(Ok(sender))
                })
                .then(move |result| {
                    // TODO: send EOF flag
                    if let Err(e) = result {
                        sender.send(ReadToWriteMessage {
                            stream_id: stream_id,
                            content: Err(e),
                        }).unwrap();
                    }

                    futures::done::<_, GrpcError>(Ok(()))
                })
                .forget();
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
        }
	}
}

struct ServerSharedState {
    conn: HttpConnection,
    factory: GrpcStreamFactory,
    state: DefaultSessionState<Server, GrpcHttp2ServerStream>,
}

struct ReadToWriteMessage {
    stream_id: StreamId,
    content: GrpcResult<Vec<u8>>,
}

fn run_read(
    read: TaskIoRead<TcpStream>,
    shared: TaskDataMutex<ServerSharedState>)
        -> GrpcFuture<()>
{
    let stream = stream::iter(iter::repeat(()).map(|x| Ok(x)));

    let future = stream.fold((read, shared), |(read, shared), _| {
        recv_raw_frame(read)
            .map(|(read, raw_frame)| {
                println!("received frame");

                shared.with(|shared: &mut ServerSharedState| {

                    let mut send_buf = VecSendFrame(Vec::new());

                    let mut session = ServerSession::new(&mut shared.state, &mut shared.factory, &mut send_buf);
                    shared.conn.handle_next_frame(
                        &mut OnceReceiveFrame::new(raw_frame),
                        &mut session).unwrap();

                    // TODO: process send
                });
                (read, shared)
            })
    });

    future
        .map(|_| ())
        .boxed()
}

fn run_write(
    write: TaskIoWrite<TcpStream>,
    shared: TaskDataMutex<ServerSharedState>,
    receiver: Receiver<ReadToWriteMessage>)
        -> GrpcFuture<()>
{
    println!("run_write");
    let future = receiver.fold((write, shared), |(write, shared), message: ReadToWriteMessage| {
        println!("writer got message from reader");

        let send_buf = shared.with(move |shared: &mut ServerSharedState| {
            let mut send_buf = VecSendFrame(Vec::new());

            match message.content {
                Ok(resp_bytes) => {
                    // TODO: might be not the first message
                    shared.conn.sender(&mut send_buf).send_headers(
                        vec![
                            Header::new(b":status", b"200"),
                            Header::new(&b"content-type"[..], &b"application/grpc"[..]),
                        ],
                        message.stream_id,
                        EndStream::No).unwrap();

                    let mut body = Vec::new();
                    write_grpc_frame(&mut body, &resp_bytes);

                    shared.conn.sender(&mut send_buf).send_data(
                        DataChunk::new_borrowed(&body[..], message.stream_id, EndStream::No)).unwrap();

                    shared.conn.sender(&mut send_buf).send_headers(
                        vec![
                            Header::new(HEADER_GRPC_STATUS.as_bytes(), b"0"),
                        ],
                        message.stream_id,
                        EndStream::Yes)
                            .unwrap();

                    shared.state.remove_stream(message.stream_id).expect("unknown stream id");
                }
                Err(e) => {
                    shared.conn.sender(&mut send_buf).send_headers(
                        vec![
                            Header::new(&b":status"[..], &b"500"[..]),
                            Header::new(HEADER_GRPC_MESSAGE.as_bytes(), format!("{:?}", e).into_bytes()),
                        ],
                        message.stream_id,
                        EndStream::Yes).unwrap();
                }
            }

            send_buf.0
        });

        tokio_io::write_all(write, send_buf)
            .map(|(write, _)| (write, shared))
    });

    future
        .map(|_| ())
        .map_err(GrpcError::from)
        .boxed()
}


fn run_connection(
    socket: TcpStream,
    peer_addr: SocketAddr,
    service_defintion: Arc<ServerServiceDefinition>,
    loop_handle: LoopHandle)
    -> IoFuture<()>
{
    println!("accepted connection from {}", peer_addr);
    let handshake = server_handshake(socket).map_err(GrpcError::from);

    let loop_handle_for_channel = loop_handle.clone();
    let three = handshake.and_then(|conn| {
        let (sender, receiver_future) = loop_handle_for_channel.channel();
        receiver_future
            .map(|receiver| {
                (conn, sender, receiver)
            })
            .map_err(GrpcError::from)
    });

    let run = three.and_then(|(conn, sender, receiver_stream)| {
        let (read, write) = TaskIo::new(conn).split();

        let shared_for_read = TaskDataMutex::new(ServerSharedState {
            conn: HttpConnection::new(HttpScheme::Http),
            state: DefaultSessionState::<Server, _>::new(),
            factory: GrpcStreamFactory {
                service_definition: service_defintion,
                sender: sender,
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

    bye
        .map(|_| ())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        .boxed()
}

fn run_server_event_loop(
    listen_addr: SocketAddr,
    service_definition: ServerServiceDefinition,
    send_to_back: mpsc::Sender<LoopToServer>)
{
    let mut lp = Loop::new().expect("loop new");

    let (shutdown_tx, shutdown_rx) = lp.handle().channel();

    let shutdown_rx = shutdown_rx.map_err(GrpcError::from);

    let listen = lp.handle().tcp_listen(&listen_addr);

    let stuff = stream_repeat((Arc::new(service_definition), lp.handle()));

    let loop_run = listen.and_then(|socket| {
        let local_addr = socket.local_addr().unwrap();
        send_to_back
            .send(LoopToServer { shutdown_tx: shutdown_tx, local_addr: local_addr })
            .expect("send back");

        socket.incoming().zip(stuff).for_each(|((socket, peer_addr), (service_definition, loop_handle))| {
            run_connection(socket, peer_addr, service_definition, loop_handle).forget();
            Ok(())
        })
    }).map_err(GrpcError::from);

    let shutdown = shutdown_rx.and_then(|shutdown_rx| {
        shutdown_rx.into_future()
            .map_err(|_| GrpcError::Other("error in shutdown channel"))
            .and_then(|_| {
                // Must complete with error,
                // so `join` with this future cancels another future.
                let result: Result<(), _> = Err(GrpcError::Other("shutdown"));
                result
            })
    });

    // Wait for either completion of connection (i. e. error)
    // or shutdown signal.
    let done = loop_run.join(shutdown);

    // TODO: do not ignore error
    lp.run(done).ok();
}
