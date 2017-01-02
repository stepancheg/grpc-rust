use std::thread;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::sync::mpsc;
use std::io;

use tokio_core::reactor;
use tokio_core::net::TcpListener;

use futures;
use futures::Stream;
use futures::Future;

use solicit::http::HttpError;

use futures_misc::*;

use super::server_conn::*;
use super::http_common::*;



struct LoopToServer {
    // used only once to send shutdown signal
    shutdown_tx: futures::sync::mpsc::UnboundedSender<()>,
    local_addr: SocketAddr,
}



pub struct Http2Server {
    loop_to_server: LoopToServer,
    thread_join_handle: Option<thread::JoinHandle<()>>,
}

fn run_server_event_loop<S, F>(
    listen_addr: SocketAddr,
    serivce: F,
    send_to_back: mpsc::Sender<LoopToServer>)
where
    S : HttpService,
    F : Fn() -> S + Send + 'static,
{
    let mut lp = reactor::Core::new().expect("http2server");

    let (shutdown_tx, shutdown_rx) = futures::sync::mpsc::unbounded();

    let shutdown_rx = shutdown_rx.map_err(|()| HttpError::IoError(io::Error::new(io::ErrorKind::Other, "shutdown_rx")));

    let listen = TcpListener::bind(&listen_addr, &lp.handle()).unwrap();

    let stuff = stream_repeat(lp.handle());

    let local_addr = listen.local_addr().unwrap();
    send_to_back
        .send(LoopToServer { shutdown_tx: shutdown_tx, local_addr: local_addr })
        .expect("send back");

    let loop_run = listen.incoming().map_err(HttpError::from).zip(stuff).for_each(move |((socket, peer_addr), loop_handle)| {
        info!("accepted connection from {}", peer_addr);
        loop_handle.spawn(HttpServerConnectionAsync::new_plain(&loop_handle, socket, serivce()).map_err(|e| { warn!("connection end: {:?}", e); () }));
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
    pub fn new<S, F>(port: u16, service: F) -> Http2Server
        where
            S : HttpService,
            F : Fn() -> S + Send + 'static,
    {
        let listen_addr = ("::", port).to_socket_addrs().unwrap().next().unwrap();

        let (get_from_loop_tx, get_from_loop_rx) = mpsc::channel();

        let join_handle = thread::spawn(move || {
            run_server_event_loop(listen_addr, service, get_from_loop_tx);
        });

        let loop_to_server = get_from_loop_rx.recv().unwrap();

        Http2Server {
            loop_to_server: loop_to_server,
            thread_join_handle: Some(join_handle),
        }
    }

    pub fn local_addr(&self) -> &SocketAddr {
        &self.loop_to_server.local_addr
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

