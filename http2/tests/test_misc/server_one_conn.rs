#![allow(dead_code)]

use std::net::ToSocketAddrs;
use std::thread;
use std::io;
use std::sync::Arc;
use std::sync::Mutex;

use futures;
use futures::Future;
use futures::stream::Stream;

use native_tls::TlsAcceptor;

use tokio_core;
use tokio_core::reactor;

//use tokio_tls;

use httpbis::solicit::HttpError;
use httpbis::solicit::header::*;

use httpbis::for_test::*;


/// Single connection HTTP/server.
/// Accepts only one connection.
pub struct HttpServerOneConn {
    from_loop: FromLoop,
    join_handle: Option<thread::JoinHandle<()>>,
    shutdown_tx: Option<futures::Complete<()>>,
    conn: Arc<Mutex<Option<HttpServerConnectionAsync>>>,
}

struct FromLoop {
    port: u16,
}

impl HttpServerOneConn {
    pub fn new_fn<S>(port: u16, service: S) -> Self
        where S : Fn(Headers, HttpPartFutureStreamSend) -> HttpResponse + Send + 'static
    {
        HttpServerOneConn::new_fn_impl(port, None, service)
    }

    pub fn new_tls_fn<S>(port: u16, server_context: TlsAcceptor, service: S) -> Self
        where S : Fn(Headers, HttpPartFutureStreamSend) -> HttpResponse + Send + 'static
    {
        HttpServerOneConn::new_fn_impl(port, Some(server_context), service)
    }

    #[allow(dead_code)]
    fn new_fn_impl<S>(port: u16, server_context: Option<TlsAcceptor>, service: S) -> Self
        where S : Fn(Headers, HttpPartFutureStreamSend) -> HttpResponse + Send + 'static
    {
        let (from_loop_tx, from_loop_rx) = futures::oneshot();
        let (shutdown_tx, shutdown_rx) = futures::oneshot::<()>();

        let conn: Arc<Mutex<Option<HttpServerConnectionAsync>>> = Default::default();

        let conn_for_thread = conn.clone();

        let join_handle = thread::Builder::new().name("server_one_conn".to_owned()).spawn(move || {
            let mut lp = reactor::Core::new().unwrap();

            let listener = tokio_core::net::TcpListener::bind(&("::1", port).to_socket_addrs().unwrap().next().unwrap(), &lp.handle()).unwrap();

            let actual_port = listener.local_addr().unwrap().port();
            from_loop_tx.send(FromLoop {
                port: actual_port,
            }).ok().unwrap();

            let handle = lp.handle();

            let future = listener.incoming().into_future()
                .map_err(|_| HttpError::from(io::Error::new(io::ErrorKind::Other, "something")))
                .and_then(move |(conn, listener)| {
                    // incoming stream is endless
                    let (conn, _) = conn.unwrap();

                    // close listening port
                    drop(listener);

                    if let Some(_server_context) = server_context {
                        //HttpServerConnectionAsync::new_tls_fn(&handle, conn, server_context, service)
                        unimplemented!()
                    } else {
                        let (conn, future) = HttpServerConnectionAsync::new_plain_fn(
                            &handle, conn, Default::default(), service);
                        *conn_for_thread.lock().unwrap() = Some(conn);
                        future
                    }
                });

            let shutdown_rx = shutdown_rx.then(|_| futures::finished::<_, ()>(()));
            let future = future.then(|_| futures::finished::<_, ()>(()));

            lp.run(shutdown_rx.select(future)).ok();
        }).expect("spawn");

        HttpServerOneConn {
            from_loop: from_loop_rx.wait().unwrap(),
            join_handle: Some(join_handle),
            shutdown_tx: Some(shutdown_tx),
            conn: conn,
        }
    }
}

#[allow(dead_code)]
impl HttpServerOneConn {
    pub fn port(&self) -> u16 {
        self.from_loop.port
    }

    pub fn dump_state(&self) -> ConnectionStateSnapshot {
        let g = self.conn.lock().expect("lock");
        let conn = g.as_ref().expect("conn");
        conn.dump_state().wait().expect("dump_status")
    }
}

impl Drop for HttpServerOneConn {
    fn drop(&mut self) {
        drop(self.shutdown_tx.take().unwrap().send(()));
        self.join_handle.take().unwrap().join().ok();
    }
}
