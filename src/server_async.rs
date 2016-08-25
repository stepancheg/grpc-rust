use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::io;
use std::io::Cursor;
use std::iter::repeat;

use solicit::http::server::ServerConnection;
use solicit::http::server::StreamFactory;
use solicit::http::server::ServerSession;
use solicit::http::transport::TransportStream;
use solicit::http::transport::TransportReceiveFrame;
use solicit::http::session::Server;
use solicit::http::session::SessionState;
use solicit::http::session::DefaultSessionState;
use solicit::http::session::DefaultStream;
use solicit::http::session::StreamState;
use solicit::http::session::StreamDataChunk;
use solicit::http::session::StreamDataError;
use solicit::http::session::Stream as solicit_Stream;
use solicit::http::connection::HttpConnection;
use solicit::http::connection::SendStatus;
use solicit::http::connection::HttpFrame;
use solicit::http::frame::RawFrame;
use solicit::http::HttpScheme;
use solicit::http::StreamId;
use solicit::http::Header;
use solicit::http::HttpResult;
use solicit::http::Response;
use solicit::http::HttpError;

use futures::done;
use futures::Future;
use futures::stream;
use futures::stream::Stream;
use futures_io::IoFuture;
use futures_io::TaskIo;
use futures_io::TaskIoRead;
use futures_io::TaskIoWrite;
use futures_mio::Loop;
use futures_mio::LoopHandle;
use futures_mio::TcpStream;
use futures_mio::Sender;
use futures_mio::Receiver;

use method::*;
use result::*;
use futures_grpc::*;
use futures_misc::*;
use solicit_async::*;
use solicit_misc::*;
use grpc::*;


pub trait MethodHandlerAsync<Req, Resp> {
    fn handle(&self, req: Req) -> GrpcFuture<Resp>;
}

pub struct MethodHandlerAsyncEcho;

impl<A : Send + 'static> MethodHandlerAsync<A, A> for MethodHandlerAsyncEcho {
    fn handle(&self, req: A) -> GrpcFuture<A> {
        println!("handle echo");
        done(Ok(req)).boxed()
    }
}

pub struct MethodHandlerAsyncFn<F> {
    f: F
}

impl<F> MethodHandlerAsyncFn<F> {
    pub fn new<Req, Resp>(f: F)
        -> Self
        where F : Fn(Req) -> GrpcFuture<Resp>
    {
        MethodHandlerAsyncFn {
            f: f,
        }
    }
}

impl<Req, Resp, F> MethodHandlerAsync<Req, Resp> for MethodHandlerAsyncFn<F>
    where
        Resp : Send + 'static,
        F : Fn(Req) -> GrpcFuture<Resp>,
{
    fn handle(&self, req: Req) -> GrpcFuture<Resp> {
        (self.f)(req)
    }
}

trait MethodHandlerDispatchAsync {
    fn on_message(&self, message: &[u8]) -> GrpcFuture<Vec<u8>>;
}

struct MethodHandlerDispatchAsyncImpl<Req, Resp> {
    desc: Arc<MethodDescriptor<Req, Resp>>,
    method_handler: Box<MethodHandlerAsync<Req, Resp> + Sync + Send>,
}

impl<Req, Resp> MethodHandlerDispatchAsync for MethodHandlerDispatchAsyncImpl<Req, Resp>
    where
        Req : Send + 'static,
        Resp : Send + 'static,
{
    fn on_message(&self, message: &[u8]) -> GrpcFuture<Vec<u8>> {
        let req = self.desc.req_marshaller.read(message);
        let resp = self.method_handler.handle(req);
        let desc_copy = self.desc.clone();
        resp
            .map(move |resp| desc_copy.resp_marshaller.write(&resp))
            .boxed()
    }
}

pub struct ServerMethodAsync {
    name: String,
    dispatch: Box<MethodHandlerDispatchAsync + Sync + Send>,
}

impl ServerMethodAsync {
    pub fn new<Req, Resp, H>(method: MethodDescriptor<Req, Resp>, handler: H) -> ServerMethodAsync
        where
            Req : Send + 'static,
            Resp : Send + 'static,
            H : MethodHandlerAsync<Req, Resp> + 'static + Sync + Send,
    {
        ServerMethodAsync {
            name: method.name.clone(),
            dispatch: Box::new(MethodHandlerDispatchAsyncImpl {
                desc: Arc::new(method),
                method_handler: Box::new(handler),
            }),
        }
    }
}

pub struct ServerServiceDefinitionAsync {
    methods: Vec<ServerMethodAsync>,
}

impl ServerServiceDefinitionAsync {
    pub fn new(mut methods: Vec<ServerMethodAsync>) -> ServerServiceDefinitionAsync {
        ServerServiceDefinitionAsync {
            methods: methods,
        }
    }

    pub fn find_method(&self, name: &str) -> &ServerMethodAsync {
        self.methods.iter()
            .filter(|m| m.name == name)
            .next()
            .expect(&format!("unknown method: {}", name))
    }

    pub fn handle_method(&self, name: &str, message: &[u8]) -> GrpcFuture<Vec<u8>> {
        self.find_method(name).dispatch.on_message(message)
    }
}



pub struct GrpcServerAsync {

}

impl GrpcServerAsync {
    pub fn new(port: u16, service_definition: ServerServiceDefinitionAsync) -> GrpcServerAsync {
        let listen_addr = ("::", port).to_socket_addrs().unwrap().next().unwrap();

        thread::spawn(move || {
            run_server_event_loop(listen_addr, service_definition);
        });

        GrpcServerAsync {
        }
    }
}

struct GrpcHttp2ServerStream {
    stream: DefaultStream,
    service_definition: Arc<ServerServiceDefinitionAsync>,
    path: String,
    loop_handle: LoopHandle,
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

            // TODO: https://github.com/alexcrichton/futures-rs/issues/99
            let sender = Mutex::new(self.sender.clone());
            self.service_definition.handle_method(&self.path, message)
                .map(move |resp_bytes| {
                    println!("send to writer...");
                    sender.lock().unwrap().send(ReadToWriteMessage {
                        stream_id: stream_id,
                        resp_bytes: resp_bytes,
                    }).unwrap();
                    println!("send to writer done");
                    ()
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
    loop_handle: LoopHandle,
    service_definition: Arc<ServerServiceDefinitionAsync>,
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
            loop_handle: self.loop_handle.clone(),
            sender: self.sender.clone(),
        }
	}
}

struct ServerSharedState {
    conn: ServerConnection<GrpcStreamFactory, DefaultSessionState<Server, GrpcHttp2ServerStream>>,
}

struct ReadToWriteMessage {
    stream_id: StreamId,
    resp_bytes: Vec<u8>,
}

fn run_read(
    read: TaskIoRead<TcpStream>,
    shared: TaskDataMutex<ServerSharedState>)
        -> GrpcFuture<()>
{
    let stream = stream::iter(repeat(()).map(|x| Ok(x)));

    let future = stream.fold((read, shared), |(read, shared), _| {
        recv_raw_frame(read)
            .map(|(read, raw_frame)| {
                println!("received frame");

                shared.with(|shared| {

                    let mut send = VecSendFrame(Vec::new());

                    shared.conn.handle_next_frame(
                        &mut OnceReceiveFrame::new(raw_frame),
                        &mut send);

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
    let future = receiver.fold(write, |write, message| {
        println!("writer got message from reader");
        future_success::<_, io::Error>(write)
    });

    future
        .map(|_| {
            println!("write done");
            ()
        })
        .map_err(GrpcError::from)
        .boxed()
}


fn run_connection(
    socket: TcpStream,
    peer_addr: SocketAddr,
    service_defintion: Arc<ServerServiceDefinitionAsync>,
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
            conn: ServerConnection::with_connection(
                HttpConnection::new(HttpScheme::Http),
                DefaultSessionState::<Server, _>::new(),
                GrpcStreamFactory {
                    loop_handle: loop_handle,
                    service_definition: service_defintion,
                    sender: sender,
                }
            ),
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

fn run_server_event_loop(listen_addr: SocketAddr, service_definition: ServerServiceDefinitionAsync) {
    let mut lp = Loop::new().unwrap();

    let listen = lp.handle().tcp_listen(&listen_addr);

    let stuff = stream_repeat((Arc::new(service_definition), lp.handle()));

    let done = listen.and_then(|socket| {
        socket.incoming().zip(stuff).for_each(|((socket, peer_addr), (service_definition, loop_handle))| {
            run_connection(socket, peer_addr, service_definition, loop_handle).forget();
            Ok(())
        })
    });

    lp.run(done).unwrap();
}
