use std::thread;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::sync::mpsc;
use std::sync::Arc;
use std::str::from_utf8;

use futures;
use futures::Future;
use futures::stream::Stream;

use tokio_core;
use tokio_core::reactor;

use solicit::http::HttpScheme;
use solicit::http::HttpError;
use solicit::http::Header;
use solicit::http::StaticHeader;


use method::MethodDescriptor;

use error::*;
use result::*;

use futures_misc::*;
use futures_grpc::*;

use grpc::*;

use http_client::*;

use assert_types::*;


trait GrpcResponseHandlerTrait : Send + 'static + HttpClientResponseHandler {
}

struct GrpcResponseHandlerTyped<Req : Send + 'static, Resp : Send + 'static> {
    method: Arc<MethodDescriptor<Req, Resp>>,
    complete: tokio_core::channel::Sender<ResultOrEof<Resp, GrpcError>>,
    remaining_response: Vec<u8>,
}

impl<Req : Send + 'static, Resp : Send + 'static> GrpcResponseHandlerTrait for GrpcResponseHandlerTyped<Req, Resp> {
}

impl<Req : Send + 'static, Resp : Send + 'static> HttpClientResponseHandler for GrpcResponseHandlerTyped<Req, Resp> {
    fn headers(&mut self, headers: Vec<StaticHeader>) -> bool {
        println!("client: received headers");
        if slice_get_header(&headers, ":status") != Some("200") {
            if let Some(message) = slice_get_header(&headers, HEADER_GRPC_MESSAGE) {
                self.complete.send(ResultOrEof::Error(GrpcError::GrpcMessage(GrpcMessageError { grpc_message: message.to_owned() }))).unwrap();
            } else {
                self.complete.send(ResultOrEof::Error(GrpcError::Other("not 200"))).unwrap();
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
                    self.complete.send(ResultOrEof::Error(e)).unwrap();
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
        let _status_200 = slice_get_header(&headers, ":status") == Some("200");
        let grpc_status_0 = slice_get_header(&headers, HEADER_GRPC_STATUS) == Some("0");
        if /* status_200 && */ grpc_status_0 {
            true
        } else {
            if let Some(message) = slice_get_header(&headers, HEADER_GRPC_MESSAGE) {
                self.complete.send(ResultOrEof::Error(GrpcError::GrpcMessage(GrpcMessageError { grpc_message: message.to_owned() }))).unwrap();
            } else {
                self.complete.send(ResultOrEof::Error(GrpcError::Other("not xxx"))).unwrap();
            }
            false
        }
    }

    fn end(&mut self) {
        self.complete.send(ResultOrEof::Eof).unwrap();
    }
}

struct GrpcResponseHandler {
    tr: Box<GrpcResponseHandlerTrait>,
}

impl HttpClientResponseHandler for GrpcResponseHandler {
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
    shutdown_tx: tokio_core::channel::Sender<()>,
    loop_handle: reactor::Remote,
    http_conn: Arc<HttpClientConnectionAsync<GrpcResponseHandler>>,
}

fn _assert_loop_to_client() {
    assert_send::<reactor::Remote>();
    assert_send::<HttpClientConnectionAsync<GrpcResponseHandler>>();
    assert_send::<HttpClientConnectionAsync<GrpcResponseHandler>>();
    assert_sync::<HttpClientConnectionAsync<GrpcResponseHandler>>();
    assert_send::<Arc<HttpClientConnectionAsync<GrpcResponseHandler>>>();
    assert_send::<tokio_core::channel::Sender<()>>();
    assert_send::<LoopToClient>();
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
        -> futures::Oneshot<(tokio_core::channel::Sender<ResultOrEof<Resp, GrpcError>>, GrpcStreamSend<Resp>)>
    {
        let (one_sender, one_receiver) = futures::oneshot();

        self.loop_to_client.loop_handle.spawn(move |handle| {
            let (sender, receiver) = tokio_core::channel::channel(&handle).unwrap();
            let receiver: GrpcStreamSend<ResultOrEof<Resp, GrpcError>> = Box::new(receiver.map_err(GrpcError::from));

            let receiver: GrpcStreamSend<Resp> = Box::new(stream_with_eof_and_error(receiver));

            one_sender.complete((sender, receiver));

            futures::finished(())
        });

        one_receiver
    }

    pub fn call_impl<Req : Send + 'static, Resp : Send + 'static>(&self, req: GrpcStreamSend<Req>, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcStreamSend<Resp>
    {
        let host = self.host.clone();
        let http_scheme = self.http_scheme.clone();
        let http_conn = self.loop_to_client.http_conn.clone();

        // A channel to send response back to caller
        let future = self.new_resp_channel().map_err(GrpcError::from).and_then(move |(complete, receiver)| {

            let headers = vec![
                Header::new(":method", "POST"),
                Header::new(":path", method.name.clone()),
                Header::new(":authority", host.clone()),
                Header::new(":scheme", http_scheme.as_bytes()),
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

            let start_request = http_conn.start_request(
                headers,
                Box::new(request_frames),
                GrpcResponseHandler {
                    tr: Box::new(GrpcResponseHandlerTyped {
                        method: method.clone(),
                        complete: complete,
                        remaining_response: Vec::new(),
                    }),
                }
            ).map_err(GrpcError::from);

            let receiver: GrpcStreamSend<Resp> = receiver;

            start_request.map(move |()| receiver)
        });

        let s: GrpcStreamSend<Resp> = future_flatten_to_stream(future);
        s
    }

    pub fn call_unary<Req : Send + 'static, Resp : Send + 'static>(&self, req: Req, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcFutureSend<Resp>
    {
        stream_single_send(self.call_impl(Box::new(stream_once_send(req)), method))
    }

    pub fn call_server_streaming<Req : Send + 'static, Resp : Send + 'static>(&self, req: Req, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcStreamSend<Resp>
    {
        self.call_impl(stream_once_send(req).boxed(), method)
    }

    pub fn call_client_streaming<Req : Send + 'static, Resp : Send + 'static>(&self, req: GrpcStreamSend<Req>, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcFutureSend<Resp>
    {
        stream_single_send(self.call_impl(req, method))
    }

    pub fn call_bidi<Req : Send + 'static, Resp : Send + 'static>(&self, req: GrpcStreamSend<Req>, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcStreamSend<Resp>
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


// Event loop entry point
fn run_client_event_loop(
    socket_addr: SocketAddr,
    send_to_back: mpsc::Sender<LoopToClient>)
{
    // Create an event loop.
    let mut lp = reactor::Core::new().unwrap();

    // Create a channel to receive shutdown signal.
    let (shutdown_tx, shutdown_rx) = tokio_core::channel::channel(&lp.handle()).unwrap();

    let (http_conn, http_conn_future) = HttpClientConnectionAsync::new(lp.handle(), &socket_addr);
    let http_conn_future: GrpcFuture<_> = Box::new(http_conn_future.map_err(GrpcError::from));

    // Send channels back to GrpcClient
    send_to_back
        .send(LoopToClient {
            shutdown_tx: shutdown_tx,
            loop_handle: lp.remote(),
            http_conn: Arc::new(http_conn),
        })
        .expect("send back");

    let shutdown = shutdown_rx.into_future().map_err(|(e, _)| GrpcError::from(e)).and_then(move |_| {
        // Must complete with error,
        // so `join` with this future cancels another future.
        futures::failed::<(), _>(GrpcError::Other("shutdown"))
    });

    // Wait for either completion of connection (i. e. error)
    // or shutdown signal.
    let done = http_conn_future.join(shutdown);

    // TODO: do not ignore error
    lp.run(done).ok();
}
