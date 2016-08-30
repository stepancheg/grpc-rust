use std::thread;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::sync::mpsc;
use std::sync::Arc;
use std::str::from_utf8;

use futures;
use futures::Future;
use futures::stream::Stream;

use tokio_core::io as tokio_io;
use tokio_core::io::TaskIo;
use tokio_core::io::TaskIoRead;
use tokio_core::io::TaskIoWrite;

use tokio_core;
use tokio_core::Loop;
use tokio_core::LoopHandle;
use tokio_core::TcpStream;

use solicit::http::client::ClientConnection;
use solicit::http::client::RequestStream;
use solicit::http::session::Client;
use solicit::http::session::SessionState;
use solicit::http::session::DefaultSessionState;
use solicit::http::session::DefaultStream;
use solicit::http::session::StreamState;
use solicit::http::session::StreamDataChunk;
use solicit::http::session::StreamDataError;
use solicit::http::session::Stream as solicit_Stream;
use solicit::http::connection::HttpConnection;
use solicit::http::connection::SendStatus;
use solicit::http::HttpScheme;
use solicit::http::Header;


use method::MethodDescriptor;

use error::*;
use result::*;

use futures_misc::*;
use futures_grpc::*;
use futures_stream_merge2;

use grpc::*;
use solicit_async::*;
use solicit_misc::*;


// Data sent from event loop to GrpcClient
struct LoopToClient {
    // send requests to client through this channel
    call_tx: tokio_core::Sender<Box<CallRequest>>,
    // used only once to send shutdown signal
    shutdown_tx: tokio_core::Sender<()>,
    loop_handle: LoopHandle,
}

/// gRPC client implementation.
/// Used by generated code.
pub struct GrpcClient {
    loop_to_client: LoopToClient,
    thread_join_handle: Option<thread::JoinHandle<()>>,
}

trait CallRequest : Send {
    // serialize message contained in this request
    fn write_req(&self) -> GrpcResult<Vec<u8>>;
    // gRPC method name (path)
    fn method_name(&self) -> &str;
    // called when server receives response from server
    // to send it back method called
    fn process_response(&mut self, message: GrpcResult<&[u8]>);
}

struct CallRequestTyped<Req, Resp> {
    // this call method descriptor
    method: Arc<MethodDescriptor<Req, Resp>>,
    // the request
    req: Req,
    // channel to send response back to called
    complete: tokio_core::Sender<GrpcResult<Resp>>,
}

impl<Req : Send + 'static, Resp : Send + 'static> CallRequest for CallRequestTyped<Req, Resp> {
    fn write_req(&self) -> GrpcResult<Vec<u8>> {
        self.method.req_marshaller.write(&self.req)
    }

    fn method_name(&self) -> &str {
        &self.method.name
    }

    fn process_response(&mut self, message: GrpcResult<&[u8]>) {
        let resp = message.and_then(|message| self.method.resp_marshaller.read(&message));
        self.complete.send(resp).ok();
    }
}

impl GrpcClient {
    /// Create a client connected to specified host and port.
    pub fn new(host: &str, port: u16) -> GrpcResult<GrpcClient> {

        // TODO: sync
        // TODO: try connect to all addrs
        let socket_addr = try!((host, port).to_socket_addrs()).next().unwrap();

        // We need some data back from event loop.
        // This channel is used to exchange that data
        let (get_from_loop_tx, get_from_loop_rx) = mpsc::channel();

        let host = host.to_owned();

        // Start event loop.
        let join_handle = thread::spawn(move || {
            run_client_event_loop(host, socket_addr, get_from_loop_tx);
        });

        // Get back call channel and shutdown channel.
        let loop_to_client = try!(get_from_loop_rx.recv()
            .map_err(|_| GrpcError::Other("get response from loop")));

        Ok(GrpcClient {
            loop_to_client: loop_to_client,
            thread_join_handle: Some(join_handle),
        })
    }

    pub fn new_resp_channel<Resp : Send + 'static>(&self) -> (tokio_core::Sender<GrpcResult<Resp>>, GrpcStream<Resp>) {
        let (sender, receiver) = self.loop_to_client.loop_handle.clone().channel();
        let receiver: GrpcStream<Resp> = future_flatten_to_stream(receiver)
            .map_err(GrpcError::from)
            .and_then(|r| r).boxed();
        (sender, receiver)
    }

    pub fn call_unary<Req : Send + 'static, Resp : Send + 'static>(&self, req: Req, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcFuture<Resp>
    {
        single_element(self.call_server_streaming(req, method))
    }

    pub fn call_server_streaming<Req : Send + 'static, Resp : Send + 'static>(&self, req: Req, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcStream<Resp>
    {
        // A channel to send response back to caller
        let (complete, receiver) = self.new_resp_channel();

        // Send call to event loop.
        let send = self.loop_to_client.call_tx.send(Box::new(CallRequestTyped {
            method: method,
            req: req,
            complete: complete,
        }));
        match send {
            Ok(..) => {},
            Err(e) => return err_stream(GrpcError::from(e)),
        }

        receiver
    }
}

// We shutdown client in destructor.
impl Drop for GrpcClient {
    fn drop(&mut self) {
        // ignore error because even loop may be already dead
        self.loop_to_client.shutdown_tx.send(()).ok();

        // do not ignore errors because we own event loop thread
        self.thread_join_handle.take().expect("handle.take")
            .join().expect("join thread");
    }
}

struct GrpcHttp2ClientStream {
    stream: DefaultStream,
    call: Option<Box<CallRequest>>,
}

fn get_header<'a>(stream: &'a DefaultStream, name: &str) -> Option<&'a str> {
    stream.headers.as_ref()
        .and_then(|headers| {
            headers.iter()
                .find(|h| h.name() == name.as_bytes())
                .and_then(|h| from_utf8(h.value()).ok())
        })
}

impl GrpcHttp2ClientStream {
    fn new() -> GrpcHttp2ClientStream {
        GrpcHttp2ClientStream {
            stream: DefaultStream::new(),
            call: None,
        }
    }

    fn process_buf(&mut self) {
        let call = self.call.as_mut().unwrap();
        if let Some("200") = get_header(&self.stream, ":status") {
            loop {
                let len = match parse_grpc_frame(&self.stream.body) {
                    Err(e) => {
                        call.process_response(Err(e));
                        break;
                    }
                    Ok(None) => break,
                    Ok(Some((message, len))) => {
                        call.process_response(Ok(message));
                        len
                    }
                };
                self.stream.body.drain(..len);
            }
        }
    }
}

impl solicit_Stream for GrpcHttp2ClientStream {
    fn new_data_chunk(&mut self, data: &[u8]) {
        self.stream.body.extend(data.into_iter());

        self.process_buf();
    }

    fn set_headers<'n, 'v>(&mut self, headers: Vec<Header<'n, 'v>>) {
        self.stream.set_headers(headers)
    }

    fn set_state(&mut self, state: StreamState) {
        if state == StreamState::Closed {
            let status_200 = get_header(&self.stream, ":status") == Some("200");
            let grpc_status_0 = get_header(&self.stream, HEADER_GRPC_STATUS) == Some("0");
            if status_200 && grpc_status_0 {
                self.process_buf();
                if !self.stream.body.is_empty() {
                    let call = self.call.as_mut().unwrap();
                    call.process_response(Err(GrpcError::Other("partial frame")));
                }
            } else if let Some(message) = get_header(&self.stream, HEADER_GRPC_MESSAGE) {
                let call = self.call.as_mut().unwrap();
                call.process_response(Err(GrpcError::GrpcMessage(GrpcMessageError { grpc_message: message.to_owned() })));
            } else {
                let call = self.call.as_mut().unwrap();
                call.process_response(Err(GrpcError::Other("not 200/0")));
            };
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

struct ClientSharedState {
    host: String,
    conn: ClientConnection<DefaultSessionState<Client, GrpcHttp2ClientStream>>,
}


impl ClientSharedState {
    fn new_stream<'n, 'v>(
        &self,
        method: &'v [u8],
        path: &'v [u8],
        extras: &[Header<'n, 'v>],
        body: Option<Vec<u8>>)
            -> RequestStream<'n, 'v, GrpcHttp2ClientStream>
    {
        let mut stream = GrpcHttp2ClientStream::new();
        match body {
            Some(body) => stream.stream.set_full_data(body),
            None => stream.close_local(),
        };

        let mut headers: Vec<Header> = vec![
            Header::new(b":method", method),
            Header::new(b":path", path),
            Header::new(b":authority", self.host.clone().into_bytes()),
            Header::new(b":scheme", self.conn.scheme().as_bytes().to_vec()),
        ];

        headers.extend(extras.iter().cloned());

        RequestStream {
            headers: headers,
            stream: stream,
        }
    }
}

struct ReadLoopBody {
    read: TaskIoRead<TcpStream>,
    shared: TaskDataMutex<ClientSharedState>,
    read_to_write_tx: tokio_core::Sender<ReadToWrite>,
}

impl ReadLoopBody {
    fn read_and_process_frame(self) -> GrpcFuture<Self> {
        let shared = self.shared;
        let read_to_write_tx = self.read_to_write_tx;
        recv_raw_frame(self.read).map(move |(read, raw_frame)| {

            shared.with(|shared: &mut ClientSharedState| {
                let mut send = VecSendFrame(Vec::new());

                shared.conn.handle_next_frame(
                    &mut OnceReceiveFrame::new(raw_frame),
                    &mut send).unwrap();

                shared.conn.state.get_closed();

                if !send.0.is_empty() {
                    read_to_write_tx.send(ReadToWrite::SendToSocket(send.0))
                        .expect("read to write");
                }
            });

            ReadLoopBody { read: read, shared: shared, read_to_write_tx: read_to_write_tx }
        }).map_err(GrpcError::from).boxed()
    }
}

// All read from the socket happens in this function.
fn run_read(
    read: TaskIoRead<TcpStream>,
    shared: TaskDataMutex<ClientSharedState>,
    read_to_write_tx: tokio_core::Sender<ReadToWrite>)
        -> GrpcFuture<()>
{
    let stream = stream_repeat(());

    let loop_body = ReadLoopBody { read: read, shared: shared, read_to_write_tx: read_to_write_tx };

    let future = stream.fold(loop_body, |loop_body, _| {
        loop_body.read_and_process_frame()
    });

    future.map(|_| ()).boxed()
}

enum ReadToWrite {
    SendToSocket(Vec<u8>),
}


struct WriteLoopBody {
    write: TaskIoWrite<TcpStream>,
    shared: TaskDataMutex<ClientSharedState>,
}


impl WriteLoopBody {
    fn process_call(self, req: Box<CallRequest>) -> GrpcFuture<Self> {
        let send_buf: GrpcResult<Vec<u8>> = self.shared.with(|shared: &mut ClientSharedState| {
            let mut body = Vec::new();
            write_grpc_frame(&mut body, &try!(req.write_req()));
            let path = req.method_name().as_bytes().to_vec();
            let mut stream = shared.new_stream(
                b"POST",
                &path,
                &[],
                Some(body));

            stream.stream.call = Some(req);

            let mut send_buf = VecSendFrame(Vec::new());

            // TODO: do not unwrap
            let _stream_id = shared.conn.start_request(stream, &mut send_buf).unwrap();
            while let SendStatus::Sent = shared.conn.send_next_data(&mut send_buf).unwrap() {
            }

            Ok(send_buf.0)
        });

        let write = self.write;
        let shared = self.shared;
        futures::done(send_buf)
            .and_then(move |send_buf| {
                tokio_io::write_all(write, send_buf)
                    .map(move |(write, _)| WriteLoopBody { write: write, shared: shared })
                    .map_err(GrpcError::from)
            })
            .boxed()
    }

    fn process_read_to_write(self, read_to_write: ReadToWrite) -> GrpcFuture<Self> {
        let shared = self.shared;

        match read_to_write {
            ReadToWrite::SendToSocket(buf) => {
                tokio_io::write_all(self.write, buf)
                    .map(move |(write, _)| WriteLoopBody { write: write, shared: shared })
                    .map_err(GrpcError::from)
                    .boxed()
            }
        }
    }
}


// All write to socket happens in this function.
fn run_write(
    write: TaskIoWrite<TcpStream>,
    shared: TaskDataMutex<ClientSharedState>,
    call_rx: tokio_core::Receiver<Box<CallRequest>>,
    read_to_write_rx: tokio_core::Receiver<ReadToWrite>)
    -> GrpcFuture<()>
{

    let merge = futures_stream_merge2::new(call_rx, read_to_write_rx)
        .map_err(GrpcError::from);

    let loop_body = WriteLoopBody {
        write: write,
        shared: shared,
    };

    let future = merge.fold(loop_body, |loop_body: WriteLoopBody, req| {
        match req {
            futures_stream_merge2::MergedItem::First(call) => {
                loop_body.process_call(call)
            }
            futures_stream_merge2::MergedItem::Second(read_to_write) => {
                loop_body.process_read_to_write(read_to_write)
            }
        }
    });
    future
        .map(|_| ())
        .boxed()
}

// Event loop entry point
fn run_client_event_loop(
    host: String,
    socket_addr: SocketAddr,
    send_to_back: mpsc::Sender<LoopToClient>)
{
    // Create an event loop.
    let mut lp = Loop::new().unwrap();

    // Create a channel to receive method calls.
    let (call_tx, call_rx) = lp.handle().channel();
    let call_rx = call_rx.map_err(GrpcError::from);

    // Create a channel to receive shutdown signal.
    let (shutdown_tx, shutdown_rx) = lp.handle().channel();

    // Send channels back to GrpcClient
    send_to_back
        .send(LoopToClient {
            call_tx: call_tx,
            shutdown_tx: shutdown_tx,
            loop_handle: lp.handle(),
        })
        .expect("send back");

    let shutdown_rx = shutdown_rx.map_err(GrpcError::from);

    let handshake = connect_and_handshake(lp.handle(), &socket_addr)
        .map_err(GrpcError::from);

    let (read_to_write_tx, read_to_write_rx) = lp.handle().channel();
    let read_to_write_rx = read_to_write_rx.map_err(GrpcError::from);

    let run_read_write = handshake.join3(call_rx, read_to_write_rx).and_then(|(conn, call_rx, read_to_write_rx)| {
        let (read, write) = TaskIo::new(conn).split();

        let conn = HttpConnection::new(HttpScheme::Http);
        let state = DefaultSessionState::<Client, _>::new();

        let conn = ClientConnection::with_connection(conn, state);

        let shared_for_read = TaskDataMutex::new(ClientSharedState {
            host: host,
            conn: conn,
        });
        let shared_for_write = shared_for_read.clone();
        run_read(read, shared_for_read, read_to_write_tx)
            .join(run_write(write, shared_for_write, call_rx, read_to_write_rx))
    });

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
    let done = run_read_write.join(shutdown);

    // TODO: do not ignore error
    lp.run(done).ok();
}
