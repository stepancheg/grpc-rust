extern crate grpc;
extern crate futures;

mod test_misc;

use std::sync::Arc;

use futures::*;
use futures::stream::Stream;

use grpc::server::*;
use grpc::client::*;
use grpc::method::*;
use grpc::marshall::*;
use grpc::error::*;
use grpc::futures_grpc::*;

use test_misc::TestSync;


fn string_string_method(name: &str, client_streaming: bool, server_streaming: bool)
    -> Arc<MethodDescriptor<String, String>>
{
    Arc::new(MethodDescriptor {
       name: name.to_owned(),
       client_streaming: client_streaming,
       server_streaming: server_streaming,
       req_marshaller: Box::new(MarshallerString),
       resp_marshaller: Box::new(MarshallerString),
   })
}


struct TestCommon {
    server_streaming_sync: Arc<TestSync>,
}

impl TestCommon {
    fn new() -> TestCommon {
        TestCommon {
            server_streaming_sync: Arc::new(TestSync::new()),
        }
    }
}


struct TestServer {
    grpc_server: GrpcServer,
}

impl TestServer {
    fn new(common: Arc<TestCommon>) -> Self {
        let mut methods = Vec::new();
        methods.push(ServerMethod::new(
            string_string_method("/test/Echo", false, false),
            MethodHandlerUnary::new(|s| Ok(s).into_future().boxed()),
        ));
        methods.push(ServerMethod::new(
            string_string_method("/test/ServerStreaming", false, true),
            MethodHandlerServerStreaming::new(move |s| {
                let sync = common.server_streaming_sync.clone();
                test_misc::stream_thread_spawn_iter(move || {
                    (0..3).map(move |i| {
                        sync.take(i * 2 + 1);
                        format!("{}{}", s, i)
                    })
                })
            }),
        ));
        methods.push(ServerMethod::new(
            string_string_method("/test/Error", false, false),
            MethodHandlerUnary::new(|_| Err(GrpcError::Other("my error")).into_future().boxed()),
        ));
        methods.push(ServerMethod::new(
            string_string_method("/test/Panic", false, false),
            MethodHandlerUnary::new(|_| panic!("icnap")),
        ));
        TestServer {
            grpc_server: GrpcServer::new(
                0,
                ServerServiceDefinition::new(methods),
            ),
        }
    }
}

struct TestClient {
    grpc_client: GrpcClient,
}

impl TestClient {
    fn new(port: u16, _common: Arc<TestCommon>) -> TestClient {
        TestClient {
            grpc_client: GrpcClient::new("::", port).expect("GrpcClient::new"),
        }
    }

    fn call_unary(&self, name: &str, param: &str) -> GrpcFuture<String> {
        self.grpc_client.call_unary(param.to_owned(), string_string_method(name, false, false))
    }

    fn call_server_streaming(&self, name: &str, param: &str) -> GrpcStream<String> {
        self.grpc_client.call_server_streaming(param.to_owned(), string_string_method(name, false, true))
    }

    fn call_expect_error<F : FnOnce(&GrpcError) -> bool>(&self, name: &str, param: &str, expect: F) {
        match self.call_unary(name, param).wait() {
            Ok(r) => panic!("expecting error, got: {:?}", r),
            Err(e) => assert!(expect(&e), "wrong error: {:?}", e),
        }
    }

    fn call_expect_grpc_error<F : FnOnce(&str) -> bool>(&self, name: &str, param: &str, expect: F) {
        self.call_expect_error(name, param, |e| {
            match e {
                &GrpcError::GrpcMessage(GrpcMessageError { ref grpc_message }) if expect(&grpc_message) => true,
                _ => false,
            }
        });
    }

    fn call_expect_grpc_error_contain(&self, name: &str, param: &str, expect: &str) {
        self.call_expect_grpc_error(name, param, |m| m.find(expect).is_some());
    }
}

struct Tester {
    client: TestClient,
    _server: TestServer,
    common: Arc<TestCommon>,
}

fn client_and_server() -> Tester {
    let common = Arc::new(TestCommon::new());

    let server = TestServer::new(common.clone());

    let client = TestClient::new(server.grpc_server.local_addr().port(), common.clone());

    Tester {
        client: client,
        _server: server,
        common: common,
    }
}

#[test]
fn unary() {
    let tester = client_and_server();

    assert_eq!("aa", tester.client.call_unary("/test/Echo", "aa").wait().unwrap());
}

#[test]
fn server_streaming() {
    let tester = client_and_server();

    let mut rs = tester.client.call_server_streaming("/test/ServerStreaming", "x").wait();

    tester.common.server_streaming_sync.take(0);
    assert_eq!("x0", rs.next().unwrap().unwrap());
    tester.common.server_streaming_sync.take(2);
    assert_eq!("x1", rs.next().unwrap().unwrap());
    tester.common.server_streaming_sync.take(4);
    assert_eq!("x2", rs.next().unwrap().unwrap());
    assert!(rs.next().is_none());
}

#[test]
fn error_in_handler() {
    let tester = client_and_server();

    tester.client.call_expect_grpc_error_contain("/test/Error", "aa", "my error");
}

#[test]
fn panic_in_handler() {
    let tester = client_and_server();

    tester.client.call_expect_grpc_error("/test/Panic", "aa",
        |m| m.find("panic").is_some() /* && m.find("icnap").is_some() */);
}
