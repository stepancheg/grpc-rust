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
    fn headers(&mut self, headers: Vec<StaticHeader>) -> bool {
        println!("client: received headers");
        if slice_get_header(&headers, ":status") != Some("200") {
            if let Some(message) = slice_get_header(&headers, HEADER_GRPC_MESSAGE) {
                self.complete.send(ResultOrEof::Error(GrpcError::GrpcMessage(GrpcMessageError { grpc_message: message.to_owned() })));
            } else {
                self.complete.send(ResultOrEof::Error(GrpcError::Other("not 200")));
            }
            false
        } else {
            true
        }
    }

    fn data_frame(&mut self, chunk: Vec<u8>) -> bool {
        self.remaining_response.extend(&chunk);
        loop {
            let len = match parse_grpc_frame(&self.remaining_response) {
                Err(e) => {
                    self.complete.send(ResultOrEof::Error(e));
                    return false;
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
        true
    }

    fn trailers(&mut self, headers: Vec<StaticHeader>) -> bool {
        let status_200 = slice_get_header(&headers, ":status") == Some("200");
        let grpc_status_0 = slice_get_header(&headers, HEADER_GRPC_STATUS) == Some("0");
        if /* status_200 && */ grpc_status_0 {
            true
        } else {
            if let Some(message) = slice_get_header(&headers, HEADER_GRPC_MESSAGE) {
                self.complete.send(ResultOrEof::Error(GrpcError::GrpcMessage(GrpcMessageError { grpc_message: message.to_owned() })));
            } else {
                self.complete.send(ResultOrEof::Error(GrpcError::Other("not xxx")));
            }
            false
        }
    }

    fn end(&mut self) {
        self.complete.send(ResultOrEof::Eof);
    }
}

struct GrpcResponseHandler {
    tr: Box<GrpcResponseHandlerTrait>,
}

impl HttpResponseHandler for GrpcResponseHandler {
    fn headers(&mut self, headers: Vec<StaticHeader>) -> bool {
        self.tr.headers(headers)
    }

    fn data_frame(&mut self, chunk: Vec<u8>) -> bool {
        self.tr.data_frame(chunk)
    }

    fn trailers(&mut self, headers: Vec<StaticHeader>) -> bool {
        self.tr.trailers(headers)
    }

    fn end(&mut self) {
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
    host: String,
    http_scheme: HttpScheme,
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

        // Start event loop.
        let join_handle = thread::spawn(move || {
            run_client_event_loop(socket_addr, get_from_loop_tx);
        });

        // Get back call channel and shutdown channel.
        let loop_to_client = try!(get_from_loop_rx.recv()
            .map_err(|_| GrpcError::Other("get response from loop")));

        Ok(GrpcClient {
            loop_to_client: loop_to_client,
            thread_join_handle: Some(join_handle),
            host: host.to_owned(),
            http_scheme: HttpScheme::Http,
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

    pub fn call_impl<Req : Send + 'static, Resp : Send + 'static>(&self, req: GrpcStream<Req>, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcStream<Resp>
    {
        // A channel to send response back to caller
        let (complete, receiver) = self.new_resp_channel();

        fn header(name: &str, value: &str) -> StaticHeader {
            Header::new(name.as_bytes().to_owned(), value.as_bytes().to_owned())
        }

        let headers = vec![
            header(":method", "POST"),
            header(":path", &method.name),
            header(":authority", &self.host),
            header(":scheme", from_utf8(self.http_scheme.as_bytes()).unwrap()),
        ];

        let request_frames = {
            let method = method.clone();
            req
                .and_then(move |req| {
                    let grpc_frame = try!(method.req_marshaller.write(&req));
                    Ok(write_grpc_frame_to_vec(&grpc_frame))
                })
                .map_err(|e| HttpError::Other(Box::new(e)))
        };

        let start_request = self.loop_to_client.http_conn.start_request(
            headers,
            request_frames.boxed(),
            GrpcResponseHandler {
                tr: Box::new(GrpcResponseHandlerTyped {
                    method: method.clone(),
                    complete: complete,
                    remaining_response: Vec::new(),
                }),
            }
        ).map_err(GrpcError::from);

        future_flatten_to_stream(start_request.and_then(|()| Ok(receiver))).boxed()
    }

    pub fn call_unary<Req : Send + 'static, Resp : Send + 'static>(&self, req: Req, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcFuture<Resp>
    {
        stream_single(self.call_impl(stream_once(req).boxed(), method))
    }

    pub fn call_server_streaming<Req : Send + 'static, Resp : Send + 'static>(&self, req: Req, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcStream<Resp>
    {
        self.call_impl(stream_once(req).boxed(), method)
    }

    pub fn call_client_streaming<Req : Send + 'static, Resp : Send + 'static>(&self, req: GrpcStream<Req>, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcFuture<Resp>
    {
        stream_single(self.call_impl(req, method))
    }

    pub fn call_bidi<Req : Send + 'static, Resp : Send + 'static>(&self, req: GrpcStream<Req>, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcStream<Resp>
    {
        self.call_impl(req, method)
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


// Event loop entry point
fn run_client_event_loop(
    socket_addr: SocketAddr,
    send_to_back: mpsc::Sender<LoopToClient>)
{
    // Create an event loop.
    let mut lp = Loop::new().unwrap();

    // Create a channel to receive shutdown signal.
    let (shutdown_tx, shutdown_rx) = lp.handle().channel();

    let (http_conn, http_conn_future) = HttpConnectionAsync::new(lp.handle(), &socket_addr);
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
