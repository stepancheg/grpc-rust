use std::net::ToSocketAddrs;
use std::thread;
use std::io;

use futures;
use futures::Future;
use futures::stream::Stream;

use tokio_core;
use tokio_core::reactor;

use tokio_tls;

use solicit::http::StaticHeader;
use solicit::http::HttpError;

use grpc::for_test::*;


/// Single connection HTTP/server.
/// Accepts only one connection.
#[allow(dead_code)]
pub struct HttpServerOneConn {
    from_loop: FromLoop,
    join_handle: Option<thread::JoinHandle<()>>,
    shutdown_tx: Option<futures::Complete<()>>,
}

struct FromLoop {
    port: u16,
}

impl HttpServerOneConn {
    pub fn new_fn<S>(port: u16, service: S) -> Self
        where S : Fn(Vec<StaticHeader>, HttpStreamStreamSend) -> HttpStreamStreamSend + Send + 'static
    {
        HttpServerOneConn::new_fn_impl(port, None, service)
    }

    pub fn new_tls_fn<S>(port: u16, server_context: tokio_tls::ServerContext, service: S) -> Self
        where S : Fn(Vec<StaticHeader>, HttpStreamStreamSend) -> HttpStreamStreamSend + Send + 'static
    {
        HttpServerOneConn::new_fn_impl(port, Some(server_context), service)
    }

    #[allow(dead_code)]
    fn new_fn_impl<S>(port: u16, server_context: Option<tokio_tls::ServerContext>, service: S) -> Self
        where S : Fn(Vec<StaticHeader>, HttpStreamStreamSend) -> HttpStreamStreamSend + Send + 'static
    {
        let (from_loop_tx, from_loop_rx) = futures::oneshot();
        let (shutdown_tx, shutdown_rx) = futures::oneshot::<()>();

        let join_handle = thread::spawn(move || {
            let mut lp = reactor::Core::new().unwrap();

            let listener = tokio_core::net::TcpListener::bind(&("::1", port).to_socket_addrs().unwrap().next().unwrap(), &lp.handle()).unwrap();

            let actual_port = listener.local_addr().unwrap().port();
            from_loop_tx.complete({FromLoop {
                port: actual_port,
            }});

            let handle = lp.handle();

            let future = listener.incoming().into_future()
                .map_err(|_| HttpError::from(io::Error::new(io::ErrorKind::Other, "something")))
                .and_then(move |(conn, listener)| {
                    // incoming stream is endless
                    let (conn, _) = conn.unwrap();

                    // close listening port
                    drop(listener);

                    if let Some(server_context) = server_context {
                        HttpServerConnectionAsync::new_tls_fn(&handle, conn, server_context, service)
                    } else {
                        HttpServerConnectionAsync::new_plain_fn(&handle, conn, service)
                    }
                });

            let shutdown_rx = shutdown_rx.then(|_| futures::finished::<_, ()>(()));
            let future = future.then(|_| futures::finished::<_, ()>(()));

            lp.run(shutdown_rx.select(future)).ok();
        });

        HttpServerOneConn {
            from_loop: from_loop_rx.wait().unwrap(),
            join_handle: Some(join_handle),
            shutdown_tx: Some(shutdown_tx),
        }
    }
}

#[allow(dead_code)]
impl HttpServerOneConn {
    pub fn port(&self) -> u16 {
        self.from_loop.port
    }
}

impl Drop for HttpServerOneConn {
    fn drop(&mut self) {
        self.shutdown_tx.take().unwrap().complete(());
        self.join_handle.take().unwrap().join().ok();
    }
}
