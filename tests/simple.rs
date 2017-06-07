extern crate futures;
extern crate tokio_core;
extern crate tokio_tls_api;
extern crate grpc;
#[macro_use]
extern crate log;

mod test_misc;

use std::sync::Arc;

use futures::future::*;
use futures::Sink;
use futures::stream::Stream;

use grpc::*;
use grpc::server::*;
use grpc::method::*;
use grpc::marshall::*;
use grpc::error::*;
use grpc::futures_grpc::*;

use test_misc::*;


fn string_string_method(name: &str, streaming: GrpcStreaming)
    -> Arc<MethodDescriptor<String, String>>
{
    Arc::new(MethodDescriptor {
       name: name.to_owned(),
       streaming: streaming,
       req_marshaller: Box::new(MarshallerString),
       resp_marshaller: Box::new(MarshallerString),
   })
}

/// Single method server on random port
fn new_server<H>(name: &str, handler: H) -> Server
    where
        H : MethodHandler<String, String> + 'static + Sync + Send,
        H : GrpcStreamingFlavor,
{
    let mut methods = Vec::new();
    methods.push(ServerMethod::new(
        string_string_method(name, <H as GrpcStreamingFlavor>::streaming()),
        handler,
    ));
    Server::new_plain("[::1]:0", Default::default(), ServerServiceDefinition::new(methods))
}

/// Single unary method server
fn new_server_unary<H>(name: &str, handler: H) -> Server
    where H : Fn(RequestOptions, String) -> SingleResponse<String> + Sync + Send + 'static
{
    new_server(name, MethodHandlerUnary::new(handler))
}

/// Single server streaming method server
fn new_server_server_streaming<H>(name: &str, handler: H) -> Server
    where H : Fn(RequestOptions, String) -> StreamingResponse<String> + Sync + Send + 'static
{
    new_server(name, MethodHandlerServerStreaming::new(handler))
}

/// Single server streaming method server
fn new_server_client_streaming<H>(name: &str, handler: H) -> Server
    where H : Fn(RequestOptions, StreamingRequest<String>) -> SingleResponse<String> + Sync + Send + 'static
{
    new_server(name, MethodHandlerClientStreaming::new(handler))
}


/// Tester for unary methods
struct TesterUnary {
    name: String,
    client: Client,
    _server: Server,
}

impl TesterUnary {
    fn new<H>(handler: H) -> TesterUnary
        where H : Fn(RequestOptions, String) -> SingleResponse<String> + Sync + Send + 'static
    {
        let name = "/text/Unary";
        let server = new_server_unary(name, handler);
        let port = server.local_addr().port();
        TesterUnary {
            name: name.to_owned(),
            _server: server,
            client: Client::new_plain("::1", port, Default::default()).unwrap()
        }
    }

    fn call(&self, param: &str) -> GrpcFutureSend<String> {
        self.client.call_unary(
            RequestOptions::new(),
            param.to_owned(),
            string_string_method(&self.name, GrpcStreaming::Unary))
                .drop_metadata()
    }

    fn call_expect_error<F : FnOnce(&Error) -> bool>(&self, param: &str, expect: F) {
        match self.call(param).wait() {
            Ok(r) => panic!("expecting error, got: {:?}", r),
            Err(e) => assert!(expect(&e), "wrong error: {:?}", e),
        }
    }

    fn call_expect_grpc_error<F : FnOnce(&str) -> bool>(&self, param: &str, expect: F) {
        self.call_expect_error(param, |e| {
            match e {
                &Error::GrpcMessage(GrpcMessageError { ref grpc_message, .. }) if expect(&grpc_message) => true,
                _ => false,
            }
        });
    }

    fn call_expect_grpc_error_contain(&self, param: &str, expect: &str) {
        self.call_expect_grpc_error(param, |m| m.find(expect).is_some());
    }
}


/// Tester for server streaming methods
struct TesterServerStreaming {
    name: String,
    client: Client,
    _server: Server,
}

impl TesterServerStreaming {
    fn new<H>(handler: H) -> Self
        where H : Fn(RequestOptions, String) -> StreamingResponse<String> + Sync + Send + 'static
    {
        let name = "/test/ServerStreaming";
        let server = new_server_server_streaming(name, handler);
        let port = server.local_addr().port();
        TesterServerStreaming {
            name: name.to_owned(),
            _server: server,
            client: Client::new_plain("::1", port, Default::default()).unwrap()
        }
    }

    fn call(&self, param: &str) -> GrpcStream<String> {
        self.client.call_server_streaming(
            RequestOptions::new(),
            param.to_owned(),
            string_string_method(&self.name, GrpcStreaming::ServerStreaming)).drop_metadata()
    }
}


struct TesterClientStreaming {
    name: String,
    client: Client,
    _server: Server,
}

impl TesterClientStreaming {
    fn new<H>(handler: H) -> Self
        where H : Fn(RequestOptions, StreamingRequest<String>) -> SingleResponse<String> + Sync + Send + 'static
    {
        let name = "/test/ClientStreaming";
        let server = new_server_client_streaming(name, handler);
        let port = server.local_addr().port();
        TesterClientStreaming {
            name: name.to_owned(),
            _server: server,
            client: Client::new_plain("::1", port, Default::default()).unwrap()
        }
    }

    fn call(&self) -> (futures::sync::mpsc::Sender<String>, GrpcFutureSend<String>) {
        let (tx, rx) = futures::sync::mpsc::channel(0);
        (
            tx,
            self.client.call_client_streaming(
                RequestOptions::new(),
                StreamingRequest::new(rx.map_err(|()| unreachable!())),
                string_string_method(&self.name, GrpcStreaming::ClientStreaming)).drop_metadata(),
        )
    }
}


#[test]
fn unary() {
    let tester = TesterUnary::new(|_m, s| SingleResponse::completed(s));

    assert_eq!("aa", tester.call("aa").wait().unwrap());
}

#[test]
fn server_is_not_running() {
    let client = Client::new_plain("::1", 2, Default::default()).unwrap();

    // TODO: https://github.com/tokio-rs/tokio-core/issues/12
    if false {
        let result = client.call_unary(
            RequestOptions::new(),
            "aa".to_owned(),
            string_string_method("/does/not/matter", GrpcStreaming::Unary)).wait();
        assert!(result.is_err(), result);
    }
}

#[test]
fn error_in_handler() {
    let tester = TesterUnary::new(|_m, _| SingleResponse::no_metadata(Err(Error::Other("my error")).into_future()));

    tester.call_expect_grpc_error_contain("aa", "my error");
}

// TODO
//#[test]
fn panic_in_handler() {
    let tester = TesterUnary::new(|_m, _| panic!("icnap"));

    tester.call_expect_grpc_error("aa",
        |m| m.find("Panic").is_some() && m.find("icnap").is_some());
}

#[test]
fn server_streaming() {
    let test_sync = Arc::new(TestSync::new());
    let test_sync_server = test_sync.clone();

    let tester = TesterServerStreaming::new(move |_m, s| {
        let sync = test_sync_server.clone();
        StreamingResponse::no_metadata(test_misc::stream_thread_spawn_iter(move || {
            (0..3).map(move |i| {
                sync.take(i * 2 + 1);
                format!("{}{}", s, i)
            })
        }))
    });

    let mut rs = tester.call("x").wait();

    test_sync.take(0);
    assert_eq!("x0", rs.next().unwrap().unwrap());
    test_sync.take(2);
    assert_eq!("x1", rs.next().unwrap().unwrap());
    test_sync.take(4);
    assert_eq!("x2", rs.next().unwrap().unwrap());
    assert!(rs.next().is_none());
}

#[test]
fn client_streaming() {
    let tester = TesterClientStreaming::new(move |_m, s| {
        SingleResponse::no_metadata(s.0.fold(String::new(), |mut s, message| {
            s.push_str(&message);
            futures::finished::<_, Error>(s)
        }))
    });

    let (tx, result) = tester.call();
    let tx = tx.send("aa".to_owned()).wait().ok().expect("aa");
    let tx = tx.send("bb".to_owned()).wait().ok().expect("bb");
    let tx = tx.send("cc".to_owned()).wait().ok().expect("cc");
    drop(tx);

    assert_eq!("aabbcc", result.wait().unwrap());
}
