extern crate httpbis;
extern crate futures;
extern crate tokio_core;
extern crate tokio_tls;
#[macro_use]
extern crate log;
extern crate env_logger;

use futures::Future;

mod test_misc;

use test_misc::*;

use httpbis::for_test::*;


#[test]
fn stream_count() {
    env_logger::init().unwrap();

    let server = HttpServerEcho::new();

    debug!("started server on {}", server.port);

    let client: HttpClient =
        HttpClient::new("::1", server.port, false, Default::default()).expect("connect");

    let state: ConnectionStateSnapshot = client.dump_state().wait().expect("state");
    assert_eq!(0, state.streams.len());

    let message = client.start_post_simple_response("/foobar", (b"xxyy"[..]).to_owned())
        .wait()
        .expect("r");
    assert_eq!((b"xxyy"[..]).to_owned(), message.body);

    let state: ConnectionStateSnapshot = client.dump_state().wait().expect("state");
    assert_eq!(0, state.streams.len(), "{:?}", state);
}
