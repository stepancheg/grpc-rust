use std::sync::Arc;
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
use futures_mio::TcpStream;

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
}

impl GrpcHttp2ServerStream {
    fn with_id(stream_id: StreamId, service_definition: Arc<ServerServiceDefinitionAsync>) -> Self {
        println!("new stream {}", stream_id);
        GrpcHttp2ServerStream {
            stream: DefaultStream::new(),
            service_definition: service_definition,
            path: String::new(),
        }
    }
}

impl solicit_Stream for GrpcHttp2ServerStream {
    fn set_headers(&mut self, headers: Vec<Header>) {
        self.stream.set_headers(headers)
    }

    fn new_data_chunk(&mut self, data: &[u8]) {
        self.stream.new_data_chunk(data)
    }

    fn set_state(&mut self, state: StreamState) {
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
    service_definition: Arc<ServerServiceDefinitionAsync>,
}

impl StreamFactory for GrpcStreamFactory {
	type Stream = GrpcHttp2ServerStream;

	fn create(&mut self, id: StreamId) -> GrpcHttp2ServerStream {
		GrpcHttp2ServerStream::with_id(id, self.service_definition.clone())
	}
}

struct ServerSharedState {
    conn: ServerConnection<GrpcStreamFactory, DefaultSessionState<Server, GrpcHttp2ServerStream>>,
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
    shared: TaskDataMutex<ServerSharedState>)
        -> GrpcFuture<()>
{
    done(Ok(())).boxed()
}


fn run_connection(
    socket: TcpStream,
    peer_addr: SocketAddr,
    service_defintion: Arc<ServerServiceDefinitionAsync>)
        -> IoFuture<()>
{
    println!("accepted connection from {}", peer_addr);
    let handshake = server_handshake(socket)
        .map_err(|e| e.into());

    let run = handshake.and_then(|conn| {
        let (read, write) = TaskIo::new(conn).split();

        let shared_for_read = TaskDataMutex::new(ServerSharedState {
            conn: ServerConnection::with_connection(
                HttpConnection::new(HttpScheme::Http),
                DefaultSessionState::<Server, _>::new(),
                GrpcStreamFactory {
                    // TODO
                    service_definition: service_defintion,
                }
            ),
        });
        let shared_for_write = shared_for_read.clone();

        run_read(read, shared_for_read)
            .join(run_write(write, shared_for_write))
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

    let service_definition_stream = stream_repeat(Arc::new(service_definition));

    let done = listen.and_then(|socket| {
        socket.incoming().zip(service_definition_stream).for_each(|((socket, peer_addr), service_definition)| {
            run_connection(socket, peer_addr, service_definition).forget();
            Ok(())
        })
    });

    lp.run(done).unwrap();
}
