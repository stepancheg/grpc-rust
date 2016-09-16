use std::thread;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::sync::mpsc;
use std::sync::Arc;

use futures;
use futures::Future;
use futures::stream::Stream;

use tokio_core;
use tokio_core::reactor;

use solicit::http::HttpScheme;
use solicit::http::HttpError;
use solicit::http::Header;


use method::MethodDescriptor;

use error::*;
use result::*;

use futures_misc::*;
use futures_grpc::*;

use grpc_frame::*;

use http_client::*;

use assert_types::*;


// Data sent from event loop to GrpcClient
struct LoopToClient {
    // used only once to send shutdown signal
    shutdown_tx: tokio_core::channel::Sender<()>,
    loop_handle: reactor::Remote,
    http_conn: Arc<HttpClientConnectionAsync>,
}

fn _assert_loop_to_client() {
    assert_send::<reactor::Remote>();
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

        let http_response_stream = http_conn
            .start_request(
                headers,
                Box::new(request_frames));

        let grpc_frames = GrpcFrameFromHttpFramesStreamResponse::new(http_response_stream);

        let grpc_messages = grpc_frames.and_then(move |frame| method.resp_marshaller.read(&frame));

        Box::new(grpc_messages)
    }

    pub fn call_unary<Req : Send + 'static, Resp : Send + 'static>(&self, req: Req, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcFutureSend<Resp>
    {
        Box::new(stream_single(self.call_impl(Box::new(stream_once(req)), method)))
    }

    pub fn call_server_streaming<Req : Send + 'static, Resp : Send + 'static>(&self, req: Req, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcStreamSend<Resp>
    {
        self.call_impl(stream_once(req).boxed(), method)
    }

    pub fn call_client_streaming<Req : Send + 'static, Resp : Send + 'static>(&self, req: GrpcStreamSend<Req>, method: Arc<MethodDescriptor<Req, Resp>>)
        -> GrpcFutureSend<Resp>
    {
        Box::new(stream_single(self.call_impl(req, method)))
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
