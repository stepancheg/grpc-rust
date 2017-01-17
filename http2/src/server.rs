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
use futures::Stream;
use futures::Future;
use futures::future::join_all;

use solicit::HttpError;

use futures_misc::*;
use solicit_async::*;

use super::server_conn::*;
use super::http_common::*;



struct LoopToServer {
    // used only once to send shutdown signal
    shutdown_tx: futures::sync::mpsc::UnboundedSender<()>,
    local_addr: SocketAddr,
}



pub struct Http2Server {
    state: Arc<Mutex<HttpServerState>>,
    loop_to_server: LoopToServer,
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

fn run_server_event_loop<S>(
    listen_addr: SocketAddr,
    state: Arc<Mutex<HttpServerState>>,
    service: S,
    send_to_back: mpsc::Sender<LoopToServer>)
        where S : HttpService,
{
    let service = Arc::new(service);

    let mut lp = reactor::Core::new().expect("http2server");

    let (shutdown_tx, shutdown_rx) = futures::sync::mpsc::unbounded();

    let shutdown_rx = shutdown_rx.map_err(|()| HttpError::IoError(io::Error::new(io::ErrorKind::Other, "shutdown_rx")));

    let listen = TcpListener::bind(&listen_addr, &lp.handle()).unwrap();

    let stuff = stream_repeat((lp.handle(), service, state));

    let local_addr = listen.local_addr().unwrap();
    send_to_back
        .send(LoopToServer { shutdown_tx: shutdown_tx, local_addr: local_addr })
        .expect("send back");

    let loop_run = listen.incoming().map_err(HttpError::from).zip(stuff).for_each(move |((socket, peer_addr), (loop_handle, service, state))| {
        info!("accepted connection from {}", peer_addr);

        socket.set_nodelay(true).expect("failed to set TCP_NODELAY");

        let (conn, future) = HttpServerConnectionAsync::new_plain(&loop_handle, socket, service);

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

    let shutdown = shutdown_rx.into_future().map_err(|(e, _)| HttpError::from(e)).and_then(|_| {
        // Must complete with error,
        // so `join` with this future cancels another future.
        futures::failed::<(), _>(HttpError::IoError(io::Error::new(io::ErrorKind::Other, "shutdown_rx")))
    });

    // Wait for either completion of connection (i. e. error)
    // or shutdown signal.
    let done = loop_run.join(shutdown);

    // TODO: do not ignore error
    lp.run(done).ok();
}

impl Http2Server {
    pub fn new<A: ToSocketAddrs, S>(addr: A, service: S) -> Http2Server
        where S : HttpService
    {
        let listen_addr = addr.to_socket_addrs().unwrap().next().unwrap();

        let (get_from_loop_tx, get_from_loop_rx) = mpsc::channel();

        let state: Arc<Mutex<HttpServerState>> = Default::default();

        let state_copy = state.clone();

        let join_handle = thread::spawn(move || {
            run_server_event_loop(listen_addr, state_copy, service, get_from_loop_tx);
        });

        let loop_to_server = get_from_loop_rx.recv().unwrap();

        Http2Server {
            state: state,
            loop_to_server: loop_to_server,
            thread_join_handle: Some(join_handle),
        }
    }

    pub fn local_addr(&self) -> &SocketAddr {
        &self.loop_to_server.local_addr
    }

    // for tests
    pub fn dump_state(&self) -> HttpFutureSend<HttpServerStateSnapshot> {
        let g = self.state.lock().expect("lock");
        g.snapshot()
    }
}

// We shutdown the server in the destructor.
impl Drop for Http2Server {
    fn drop(&mut self) {
        // ignore error because even loop may be already dead
        self.loop_to_server.shutdown_tx.send(()).ok();

        // do not ignore errors of take
        // ignore errors of join, it means that server event loop crashed
        drop(self.thread_join_handle.take().unwrap().join());
    }
}

