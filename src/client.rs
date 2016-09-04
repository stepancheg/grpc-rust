use std::thread;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::sync::mpsc;
use std::sync::Arc;
use std::str::from_utf8;
use std::mem;

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

use solicit::http::client::RequestStream;
use solicit::http::client::ClientSession;
use solicit::http::session::Client;
use solicit::http::session::SessionState;
use solicit::http::session::DefaultSessionState;
use solicit::http::session::DefaultStream;
use solicit::http::session::StreamState;
use solicit::http::session::StreamDataError;
use solicit::http::session::StreamDataChunk;
use solicit::http::session::Stream as solicit_Stream;
use solicit::http::connection::HttpConnection;
use solicit::http::connection::SendStatus;
use solicit::http::connection::EndStream;
use solicit::http::connection::DataChunk;
use solicit::http::priority::SimplePrioritizer;
use solicit::http::StreamId;
use solicit::http::HttpScheme;
use solicit::http::HttpError;
use solicit::http::HttpResult;
use solicit::http::Header;
use solicit::http::StaticHeader;


use method::MethodDescriptor;

use error::*;
use result::*;

use futures_misc::*;
use futures_grpc::*;

use grpc::*;
use solicit_async::*;
use solicit_misc::*;

use http_client::*;



trait GrpcResponseHandlerTrait : Send + 'static + HttpResponseHandler {
}

struct GrpcResponseHandlerTyped<Req : Send + 'static, Resp : Send + 'static> {
    method: Arc<MethodDescriptor<Req, Resp>>,
    complete: tokio_core::Sender<ResultOrEof<Resp, GrpcError>>,
    remaining_response: Vec<u8>,
}

impl<Req : Send + 'static, Resp : Send + 'static> GrpcResponseHandlerTrait for GrpcResponseHandlerTyped<Req, Resp> {
}

impl<Req : Send + 'static, Resp : Send + 'static> HttpResponseHandler for GrpcResponseHandlerTyped<Req, Resp> {
    fn headers(&mut self, headers: Vec<StaticHeader>) -> HttpResult<()> {
        println!("client: received headers");
        if slice_get_header(&headers, ":status") != Some("200") {
            if let Some(message) = slice_get_header(&headers, HEADER_GRPC_MESSAGE) {
                self.complete.send(ResultOrEof::Error(GrpcError::GrpcMessage(GrpcMessageError { grpc_message: message.to_owned() })));
                Err(HttpError::Other(Box::new(GrpcError::GrpcMessage(GrpcMessageError { grpc_message: message.to_owned() }))))
            } else {
                self.complete.send(ResultOrEof::Error(GrpcError::Other("not 200")));
                Err(HttpError::Other(Box::new(GrpcError::Other("not 200"))))
            }
        } else {
            Ok(())
        }
    }

    fn data_frame(&mut self, chunk: Vec<u8>) -> HttpResult<()> {
        self.remaining_response.extend(&chunk);
        loop {
            let len = match parse_grpc_frame(&self.remaining_response) {
                Err(e) => {
                    self.complete.send(ResultOrEof::Error(e));
                    break;
                }
                Ok(None) => break,
                Ok(Some((message, len))) => {
                    let resp = self.method.resp_marshaller.read(&message);
                    self.complete.send(From::from(resp)).ok();
                    len
                }
            };
            self.remaining_response.drain(..len);
        }
        Ok(())
    }

    fn trailers(&mut self, headers: Vec<StaticHeader>) -> HttpResult<()> {
        let status_200 = slice_get_header(&headers, ":status") == Some("200");
        let grpc_status_0 = slice_get_header(&headers, HEADER_GRPC_STATUS) == Some("0");
        if /* status_200 && */ grpc_status_0 {
            Ok(())
        } else {
            if let Some(message) = slice_get_header(&headers, HEADER_GRPC_MESSAGE) {
                self.complete.send(ResultOrEof::Error(GrpcError::GrpcMessage(GrpcMessageError { grpc_message: message.to_owned() })));
                Err(HttpError::Other(Box::new(GrpcError::GrpcMessage(GrpcMessageError { grpc_message: message.to_owned() }))))
            } else {
                self.complete.send(ResultOrEof::Error(GrpcError::Other("not xxx")));
                Err(HttpError::Other(Box::new(GrpcError::Other("not xxx"))))
            }
        }
    }

    fn end(&mut self) -> HttpResult<()> {
        self.complete.send(ResultOrEof::Eof);
        Ok(())
    }
}

struct GrpcResponseHandler {
    tr: Box<GrpcResponseHandlerTrait>,
}

impl HttpResponseHandler for GrpcResponseHandler {
    fn headers(&mut self, headers: Vec<StaticHeader>) -> HttpResult<()> {
        self.tr.headers(headers)
    }

    fn data_frame(&mut self, chunk: Vec<u8>) -> HttpResult<()> {
        self.tr.data_frame(chunk)
    }

    fn trailers(&mut self, headers: Vec<StaticHeader>) -> HttpResult<()> {
        self.tr.trailers(headers)
    }

    fn end(&mut self) -> HttpResult<()> {
        self.tr.end()
    }
}


// Data sent from event loop to GrpcClient
struct LoopToClient {
    // used only once to send shutdown signal
    shutdown_tx: tokio_core::Sender<()>,
    loop_handle: LoopHandle,
    http_conn: HttpConnectionAsync<GrpcResponseHandler>,
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
    fn process_response(&mut self, message: ResultOrEof<&[u8], GrpcError>);
}

struct CallRequestHolder {
    request: Box<CallRequest>,
}

struct CallRequestTyped<Req, Resp> {
    // this call method descriptor
    method: Arc<MethodDescriptor<Req, Resp>>,
    // the request
    req: Req,
    // channel to send response back to called
    complete: tokio_core::Sender<ResultOrEof<Resp, GrpcError>>,
}

impl<Req : Send + 'static, Resp : Send + 'static> CallRequest for CallRequestTyped<Req, Resp> {
    fn write_req(&self) -> GrpcResult<Vec<u8>> {
        self.method.req_marshaller.write(&self.req)
    }

    fn method_name(&self) -> &str {
        &self.method.name
    }

    fn process_response(&mut self, message: ResultOrEof<&[u8], GrpcError>) {
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

    pub fn new_resp_channel<Resp : Send + 'static>(&self)
        -> (tokio_core::Sender<ResultOrEof<Resp, GrpcError>>, GrpcStream<Resp>)
    {
        let (sender, receiver) = self.loop_to_client.loop_handle.clone().channel::<ResultOrEof<Resp, GrpcError>>();
        let receiver: GrpcStream<ResultOrEof<Resp, GrpcError>> = future_flatten_to_stream(receiver)
            .map_err(GrpcError::from)
            .boxed();

        let receiver: GrpcStream<Resp> = stream_with_eof_and_error(receiver).boxed();

        (sender, receiver)
    }

    pub fn call_unary<Req : Send + 'static, Resp : Send + 'static>(&self, req: Req, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcFuture<Resp>
    {
        stream_single(self.call_server_streaming(req, method))
    }

    pub fn call_server_streaming<Req : Send + 'static, Resp : Send + 'static>(&self, req: Req, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcStream<Resp>
    {
        // A channel to send response back to caller
        let (complete, receiver) = self.new_resp_channel();

        if true {
            fn header(name: &str, value: &str) -> StaticHeader {
                Header::new(name.as_bytes().to_owned(), value.as_bytes().to_owned())
            }

            let headers = vec![
                header(":method", "POST"),
                header(":path", &method.name),
                header(":authority", "localhost"), // TODO
                header(":scheme", "http"), // TODO
            ];

            let send2 = self.loop_to_client.http_conn.start_request(
                headers,
                // TODO: do not unwrap
                // TODO: serialize in event loop
                stream_once(write_grpc_frame_to_vec(&method.req_marshaller.write(&req).unwrap())),
                GrpcResponseHandler {
                    tr: Box::new(GrpcResponseHandlerTyped {
                        method: method.clone(),
                        complete: complete,
                        remaining_response: Vec::new(),
                    }),
                }
            );
        } else {
            /*
            // Send call to event loop.
            let send = self.loop_to_client.call_tx.send(Box::new(CallRequestTyped {
                method: method,
                req: req,
                complete: complete,
            }));
            match send {
                Ok(..) => {},
                Err(e) => return stream_err(GrpcError::from(e)),
            }
            */
            panic!()
        }

        receiver
    }

    pub fn call_client_streaming<Req : Send + 'static, Resp : Send + 'static>(&self, req: GrpcStream<Req>, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcFuture<Resp>
    {
        unimplemented!();
    }

    pub fn call_bidi<Req : Send + 'static, Resp : Send + 'static>(&self, req: GrpcStream<Req>, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcStream<Resp>
    {
        unimplemented!();
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

fn slice_get_header<'a>(headers: &'a [Header<'a, 'a>], name: &str) -> Option<&'a str> {
    headers.iter()
        .find(|h| h.name() == name.as_bytes())
        .and_then(|h| from_utf8(h.value()).ok())
}

fn default_stream_get_header<'a>(stream: &'a DefaultStream, name: &str) -> Option<&'a str> {
    match stream.headers {
        Some(ref v) => slice_get_header(v, name),
        None => None,
    }
}

struct GrpcHttp2ClientStream {
    stream: DefaultStream,
    call: Box<CallRequest>,
    shared: TaskRcMut<ClientSharedState>,
}

impl GrpcHttp2ClientStream {
    fn process_buf(&mut self) {
        if let Some("200") = default_stream_get_header(&self.stream, ":status") {
            loop {
                let len = match parse_grpc_frame(&self.stream.body) {
                    Err(e) => {
                        self.call.process_response(ResultOrEof::Error(e));
                        break;
                    }
                    Ok(None) => break,
                    Ok(Some((message, len))) => {
                        self.call.process_response(ResultOrEof::Item(message));
                        len
                    }
                };
                self.stream.body.drain(..len);
            }
        }
    }

}

fn write_headers(shared: &mut ClientSharedState, stream_id: StreamId, write: TaskIoWrite<TcpStream>)
    -> GrpcFuture<(TaskIoWrite<TcpStream>, StreamId)>
{
    let stream: &mut GrpcHttp2ClientStream = shared.state.get_stream_mut(stream_id).unwrap();

    let path = stream.call.method_name().as_bytes().to_vec();

    let send_buf = {

        let headers = vec![
            Header::new(b":method", b"POST"),
            Header::new(b":path", path.to_owned()),
            Header::new(b":authority", shared.host.clone().into_bytes()),
            Header::new(b":scheme", shared.conn.scheme.as_bytes().to_vec()),
        ];

        let mut send_buf = VecSendFrame(Vec::new());

        shared.conn.sender(&mut send_buf).send_headers(headers, stream_id, EndStream::No).unwrap();

        send_buf.0
    };
    tokio_io::write_all(write, send_buf)
        .map(move |(write, _buf)| (write, stream_id))
        .map_err(GrpcError::from)
        .boxed()
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
            let status_200 = default_stream_get_header(&self.stream, ":status") == Some("200");
            let grpc_status_0 = default_stream_get_header(&self.stream, HEADER_GRPC_STATUS) == Some("0");
            if status_200 && grpc_status_0 {
                self.process_buf();
                if !self.stream.body.is_empty() {
                    self.call.process_response(ResultOrEof::Error(GrpcError::Other("partial frame")));
                } else {
                    self.call.process_response(ResultOrEof::Eof);
                }
            } else if let Some(message) = default_stream_get_header(&self.stream, HEADER_GRPC_MESSAGE) {
                self.call.process_response(ResultOrEof::Error(GrpcError::GrpcMessage(GrpcMessageError { grpc_message: message.to_owned() })));
            } else {
                self.call.process_response(ResultOrEof::Error(GrpcError::Other("not 200/0")));
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
    conn: HttpConnection,
    state: DefaultSessionState<Client, GrpcHttp2ClientStream>,
}


struct ReadLoopBody {
    read: TaskIoRead<TcpStream>,
    shared: TaskRcMut<ClientSharedState>,
    read_to_write_tx: tokio_core::Sender<ReadToWrite>,
}

impl ReadLoopBody {
    fn read_and_process_frame(self) -> GrpcFuture<Self> {
        let shared = self.shared;
        let read_to_write_tx = self.read_to_write_tx;
        recv_raw_frame(self.read).map(move |(read, raw_frame)| {

            shared.with(|shared: &mut ClientSharedState| {
                let mut send = VecSendFrame(Vec::new());

                {
                    let mut session = ClientSession::new(&mut shared.state, &mut send);
                    shared.conn.handle_next_frame(&mut OnceReceiveFrame::new(raw_frame), &mut session)
                        .unwrap();
                }

                shared.state.get_closed();

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
    shared: TaskRcMut<ClientSharedState>,
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
    shared: TaskRcMut<ClientSharedState>,
}


impl WriteLoopBody {
    fn process_call(self, req: Box<CallRequest>) -> GrpcFuture<Self> {
        let write = self.write;

        let shared_for_headers = self.shared.clone();
        let shared_for_headers2 = shared_for_headers.clone();
        let shared_for_body = shared_for_headers.clone();

        let write_headers = shared_for_headers.with(move |shared: &mut ClientSharedState| {

            let mut stream = GrpcHttp2ClientStream {
                stream: DefaultStream::new(),
                call: req,
                shared: shared_for_headers2,
            };

            let stream_id = shared.state.insert_outgoing(stream);
            {
                let stream: &mut GrpcHttp2ClientStream = shared.state.get_stream_mut(stream_id).unwrap();
                stream.stream.stream_id = Some(stream_id);
            }
            write_headers(shared, stream_id, write)
        });

        write_headers.and_then(move |(write, stream_id)| {
            let shared = shared_for_body;

            let send_buf: GrpcResult<Vec<u8>> = shared.with(|shared: &mut ClientSharedState| {
                let mut send_buf = VecSendFrame(Vec::new());

                {
                    let stream: &mut GrpcHttp2ClientStream = shared.state.get_stream_mut(stream_id).unwrap();

                    let mut body = Vec::new();
                    write_grpc_frame(&mut body, &try!(stream.call.write_req()));

                    stream.stream.set_full_data(body);

                    loop {
                        // send_next_data
                        const MAX_CHUNK_SIZE: usize = 8 * 1024;
                        let mut buf = [0; MAX_CHUNK_SIZE];

                        let chunk: StreamDataChunk = stream.get_data_chunk(&mut buf).unwrap();
                        let (size, last) = match chunk {
                            StreamDataChunk::Chunk(size) => (size, EndStream::No),
                            StreamDataChunk::Last(size) => (size, EndStream::Yes),
                            StreamDataChunk::Unavailable => panic!(),
                        };

                        let data_chunk = DataChunk::new_borrowed(&buf[..size], stream_id, last);

                        shared.conn.sender(&mut send_buf).send_data(data_chunk).unwrap();

                        if last == EndStream::Yes {
                            break;
                        }
                    }
                }

                Ok(send_buf.0)
            });

            futures::done(send_buf)
                .and_then(move |send_buf| {
                    tokio_io::write_all(write, send_buf)
                        .map(move |(write, _)| WriteLoopBody { write: write, shared: shared })
                        .map_err(GrpcError::from)
                })
        }).boxed()
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
    shared: TaskRcMut<ClientSharedState>,
    call_rx: tokio_core::Receiver<Box<CallRequest>>,
    read_to_write_rx: tokio_core::Receiver<ReadToWrite>)
    -> GrpcFuture<()>
{

    let merge = stream_merge2(call_rx, read_to_write_rx)
        .map_err(GrpcError::from);

    let loop_body = WriteLoopBody {
        write: write,
        shared: shared,
    };

    let future = merge.fold(loop_body, |loop_body: WriteLoopBody, req| {
        match req {
            Merged2Item::First(call) => {
                loop_body.process_call(call)
            }
            Merged2Item::Second(read_to_write) => {
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

    // Create a channel to receive shutdown signal.
    let (shutdown_tx, shutdown_rx) = lp.handle().channel();

    let (http_conn, http_conn_future) = HttpConnectionAsync::new(lp.handle(), &socket_addr, host.clone());
    let http_conn_future = http_conn_future.map_err(GrpcError::from);

    // Send channels back to GrpcClient
    send_to_back
        .send(LoopToClient {
            shutdown_tx: shutdown_tx,
            loop_handle: lp.handle(),
            http_conn: http_conn,
        })
        .expect("send back");

    let shutdown_rx = shutdown_rx.map_err(GrpcError::from);

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
    let done = http_conn_future.join(shutdown);

    // TODO: do not ignore error
    lp.run(done).ok();
}
