extern crate http2;
extern crate futures;
extern crate tokio_core;
extern crate tokio_tls;
extern crate solicit_fork as solicit;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::net::ToSocketAddrs;
use std::thread;

use futures::Future;
use tokio_core::reactor;

mod test_misc;

use test_misc::*;

use http2::http_client::*;


#[test]
fn stream_count() {
    env_logger::init().unwrap();

    let server = HttpServerEcho::new();

    debug!("started server on {}", server.port);

    let mut client_lp = reactor::Core::new().expect("client");

    let (client, future) = HttpClientConnectionAsync::new_plain(client_lp.handle(), &("::1", server.port).to_socket_addrs().unwrap().next().unwrap());
}
