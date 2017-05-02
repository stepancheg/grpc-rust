use std::thread;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::io;

use tokio_core::reactor;
use tokio_core::net::TcpListener;

use futures;
use futures::stream;
use futures::Stream;
use futures::Future;
use futures::future::join_all;

use solicit::HttpError;

use solicit_async::*;

use futures_misc::*;

use net2;

use super::server_conn::*;
use super::http_common::*;

use server_conf::*;

pub use server_tls::ServerTlsOption;


struct LoopToServer {
    shutdown: ShutdownSignal,
    local_addr: SocketAddr,
}



pub struct HttpServer {
    state: Arc<Mutex<HttpServerState>>,
    loop_to_server: LoopToServer,
    alive_rx: mpsc::Receiver<()>,
    thread_join_handle: Option<thread::JoinHandle<()>>,
}

#[derive(Default)]
struct HttpServerState {
    last_conn_id: u64,
    conns: HashMap<u64, HttpServerConnectionAsync>,
}

impl HttpServerState {
    fn snapshot(&self) -> HttpFutureSend<HttpServerStateSnapshot> {
        let futures: Vec<_> = self.conns.iter()
            .map(|(&id, conn)| conn.dump_state().map(move |state| (id, state)))
            .collect();

        Box::new(join_all(futures)
            .map(|states| HttpServerStateSnapshot {
                conns: states.into_iter().collect(),
            }))
    }
}

pub struct HttpServerStateSnapshot {
    pub conns: HashMap<u64, ConnectionStateSnapshot>,
}

#[cfg(unix)]
fn configure_tcp(tcp: &net2::TcpBuilder, conf: &HttpServerConf) -> io::Result<()> {
    use net2::unix::UnixTcpBuilderExt;
    if let Some(reuse_port) = conf.reuse_port {
        tcp.reuse_port(reuse_port)?;
    }
    Ok(())
}

#[cfg(windows)]
fn configure_tcp(_tcp: &net2::TcpBuilder, conf: &HttpServerConf) -> io::Result<()> {
    Ok(())
}

fn listener(
    addr: &SocketAddr,
    handle: &reactor::Handle,
    conf: &HttpServerConf)
        -> io::Result<TcpListener>
{
    let listener = match *addr {
        SocketAddr::V4(_) => net2::TcpBuilder::new_v4()?,
        SocketAddr::V6(_) => net2::TcpBuilder::new_v6()?,
    };
    configure_tcp(&listener, conf)?;
    listener.reuse_address(true)?;
    listener.bind(addr)?;
    let backlog = conf.backlog.unwrap_or(1024);
    let listener = listener.listen(backlog)?;
    TcpListener::from_listener(listener, addr, handle)
}

fn run_server_event_loop<S>(
    listen_addr: SocketAddr,
    state: Arc<Mutex<HttpServerState>>,
    tls: ServerTlsOption,
    conf: HttpServerConf,
    service: S,
    send_to_back: mpsc::Sender<LoopToServer>,
    _alive_tx: mpsc::Sender<()>)
        where S : HttpService,
{
    let service = Arc::new(service);

    let mut lp = reactor::Core::new().expect("http2server");

    let (shutdown_signal, shutdown_future) = shutdown_signal();

    let listen = listener(&listen_addr, &lp.handle(), &conf).unwrap();

    let stuff = stream::repeat((lp.handle(), service, state, tls, conf));

    let local_addr = listen.local_addr().unwrap();
    send_to_back
        .send(LoopToServer { shutdown: shutdown_signal, local_addr: local_addr })
        .expect("send back");

    let loop_run = listen.incoming().map_err(HttpError::from).zip(stuff)
        .for_each(move |((socket, peer_addr), (loop_handle, service, state, tls, conf))| {
            info!("accepted connection from {}", peer_addr);

            let no_delay = conf.no_delay.unwrap_or(true);
            socket.set_nodelay(no_delay).expect("failed to set TCP_NODELAY");

            let (conn, future) = HttpServerConnectionAsync::new(&loop_handle, socket, tls, conf, service);

            let conn_id = {
                let mut g = state.lock().expect("lock");
                g.last_conn_id += 1;
                let conn_id = g.last_conn_id;
                let prev = g.conns.insert(conn_id, conn);
                assert!(prev.is_none());
                conn_id
            };

            loop_handle.spawn(future
                .then(move |r| {
                    let mut g = state.lock().expect("lock");
                    let removed = g.conns.remove(&conn_id);
                    assert!(removed.is_some());
                    r
                })
                .map_err(|e| { warn!("connection end: {:?}", e); () }));
            Ok(())
        });

    let shutdown_future = shutdown_future
        .then(move |_| {
            // Must complete with error,
            // so `join` with this future cancels another future.
            futures::failed::<(), _>(HttpError::Shutdown)
        });

    // Wait for either completion of connection (i. e. error)
    // or shutdown signal.
    let done = loop_run.join(shutdown_future);

    // TODO: do not ignore error
    lp.run(done).ok();
}

impl HttpServer {
    pub fn new<A: ToSocketAddrs, S>(addr: A, tls: ServerTlsOption, conf: HttpServerConf, service: S) -> HttpServer
        where S : HttpService
    {
        let listen_addr = addr.to_socket_addrs().unwrap().next().unwrap();

        let (get_from_loop_tx, get_from_loop_rx) = mpsc::channel();
        let (alive_tx, alive_rx) = mpsc::channel();

        let state: Arc<Mutex<HttpServerState>> = Default::default();

        let state_copy = state.clone();

        let join_handle = thread::Builder::new()
            .name(conf.thread_name.clone().unwrap_or_else(|| "http2-server-loop".to_owned()).to_string())
            .spawn(move || {
                run_server_event_loop(
                    listen_addr,
                    state_copy,
                    tls,
                    conf, service,
                    get_from_loop_tx,
                    alive_tx);
            })
            .expect("spawn");

        let loop_to_server = get_from_loop_rx.recv().unwrap();

        HttpServer {
            state: state,
            loop_to_server: loop_to_server,
            thread_join_handle: Some(join_handle),
            alive_rx: alive_rx,
        }
    }

    pub fn local_addr(&self) -> &SocketAddr {
        &self.loop_to_server.local_addr
    }

    pub fn is_alive(&self) -> bool {
        self.alive_rx.try_recv() != Err(mpsc::TryRecvError::Disconnected)
    }

    // for tests
    pub fn dump_state(&self) -> HttpFutureSend<HttpServerStateSnapshot> {
        let g = self.state.lock().expect("lock");
        g.snapshot()
    }
}

// We shutdown the server in the destructor.
impl Drop for HttpServer {
    fn drop(&mut self) {
        self.loop_to_server.shutdown.shutdown();

        // do not ignore errors of take
        // ignore errors of join, it means that server event loop crashed
        drop(self.thread_join_handle.take().unwrap().join());
    }
}

