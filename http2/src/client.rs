use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::io;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;

use bytes::Bytes;

use futures;
use futures::Future;
use futures::stream;

use tokio_core::reactor;

use native_tls::TlsConnector;

use futures_misc::*;

use solicit::header::*;
use solicit::HttpResult;
use solicit::HttpError;
use solicit::HttpScheme;

use solicit_async::*;

use client_conn::*;
use client_conf::*;
use http_common::*;

pub use client_tls::ClientTlsOption;


// Data sent from event loop to Http2Client
struct LoopToClient {
    // used only once to send shutdown signal
    shutdown: ShutdownSignal,
    _loop_handle: reactor::Remote,
    http_conn: Arc<HttpClientConnectionAsync>,
}

pub struct HttpClient {
    loop_to_client: LoopToClient,
    thread_join_handle: Option<thread::JoinHandle<()>>,
    http_scheme: HttpScheme,
}

impl HttpClient {

    pub fn new(host: &str, port: u16, tls: bool, conf: HttpClientConf) -> HttpResult<HttpClient> {
        // TODO: sync
        // TODO: try connect to all addrs
        let socket_addr = (host, port).to_socket_addrs()?.next().unwrap();

        let tls_enabled = match tls {
            true => {
                let connector = Arc::new(TlsConnector::builder().unwrap().build().unwrap());
                ClientTlsOption::Tls(host.to_owned(), connector)
            },
            false => ClientTlsOption::Plain,
        };

        HttpClient::new_expl(&socket_addr, tls_enabled, conf)
    }

    pub fn new_expl(addr: &SocketAddr, tls: ClientTlsOption, conf: HttpClientConf) -> HttpResult<HttpClient> {
        // We need some data back from event loop.
        // This channel is used to exchange that data
        let (get_from_loop_tx, get_from_loop_rx) = mpsc::channel();

        let addr = addr.clone();
        let http_scheme = tls.http_scheme();

        // Start event loop.
        let join_handle = thread::Builder::new()
            .name(conf.thread_name.clone().unwrap_or_else(|| "http2-client-loop".to_owned()).to_string())
            .spawn(move || {
                run_client_event_loop(addr, tls, conf, get_from_loop_tx);
            })
            .expect("spawn");

        // Get back call channel and shutdown channel.
        let loop_to_client = get_from_loop_rx.recv()
            .map_err(|_| HttpError::IoError(io::Error::new(io::ErrorKind::Other, "get response from loop")))?;

        Ok(HttpClient {
            loop_to_client: loop_to_client,
            thread_join_handle: Some(join_handle),
            http_scheme: http_scheme,
        })
    }

    pub fn start_request(
        &self,
        headers: Headers,
        body: HttpFutureStreamSend<Bytes>)
            -> HttpResponse
    {
        debug!("start request {:?}", headers);
        self.loop_to_client.http_conn.start_request(headers, body)
    }

    pub fn start_request_simple(
        &self,
        headers: Headers,
        body: Bytes)
            -> HttpResponse
    {
        self.start_request(
            headers,
            Box::new(stream::once(Ok(body))))
    }

    pub fn start_get(
        &self,
        path: &str,
        authority: &str)
            -> HttpResponse
    {
        let headers = Headers(vec![
            Header::new(":method", "GET"),
            Header::new(":path", path.to_owned()),
            Header::new(":authority", authority.to_owned()),
            Header::new(":scheme", self.http_scheme.as_bytes()),
        ]);
        self.start_request_simple(headers, Bytes::new())
    }

    pub fn start_post(
        &self,
        path: &str,
        authority: &str,
        body: Bytes)
            -> HttpResponse
    {
        let headers = Headers(vec![
            Header::new(":method", "POST"),
            Header::new(":path", path.to_owned()),
            Header::new(":authority", authority.to_owned()),
            Header::new(":scheme", self.http_scheme.as_bytes()),
        ]);
        self.start_request_simple(headers, body)
    }

    pub fn dump_state(&self) -> HttpFutureSend<ConnectionStateSnapshot> {
        self.loop_to_client.http_conn.dump_state()
    }
}

// Event loop entry point
fn run_client_event_loop(
    socket_addr: SocketAddr,
    tls: ClientTlsOption,
    conf: HttpClientConf,
    send_to_back: mpsc::Sender<LoopToClient>)
{
    // Create an event loop.
    let mut lp = reactor::Core::new().unwrap();

    // Create a channel to receive shutdown signal.
    let (shutdown_signal, shutdown_future) = shutdown_signal();

    let (http_conn, http_conn_future) =
        HttpClientConnectionAsync::new(lp.handle(), &socket_addr, tls, conf);

    // Send channels back to Http2Client
    send_to_back
        .send(LoopToClient {
            shutdown: shutdown_signal,
            _loop_handle: lp.remote(),
            http_conn: Arc::new(http_conn),
        })
        .expect("send back");

    let shutdown_future = shutdown_future
        .then(move |_| {
            // Must complete with error,
            // so `join` with this future cancels another future.
            futures::failed::<(), _>(HttpError::Shutdown)
        });

    // Wait for either completion of connection (i. e. error)
    // or shutdown signal.
    let done = http_conn_future.join(shutdown_future);

    match lp.run(done) {
        Ok(_) => {}
        Err(HttpError::Shutdown) => {}
        Err(e) => {
            error!("Core::run failed: {:?}", e);
        }
    }
}

// We shutdown the client in the destructor.
impl Drop for HttpClient {
    fn drop(&mut self) {
        self.loop_to_client.shutdown.shutdown();

        // do not ignore errors because we own event loop thread
        self.thread_join_handle.take().expect("handle.take")
            .join().expect("join thread");
    }
}
