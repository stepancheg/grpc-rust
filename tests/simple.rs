extern crate grpc;
extern crate futures;

use std::sync::Arc;

use grpc::server::*;
use grpc::client::*;
use grpc::method::*;
use grpc::marshall::*;
use grpc::error::*;
use grpc::futures_grpc::*;
use futures::*;

fn string_string_method(name: &str) -> Arc<MethodDescriptor<String, String>> {
    Arc::new(MethodDescriptor {
       name: name.to_owned(),
       client_streaming: false,
       server_streaming: false,
       req_marshaller: Box::new(MarshallerString),
       resp_marshaller: Box::new(MarshallerString),
   })
}

fn test_server() -> GrpcServer {
    GrpcServer::new(
        0,
        ServerServiceDefinition::new(vec![
            ServerMethod::new(
                string_string_method("/test/Echo"),
                MethodHandlerFn::new(|s| Ok(s).into_future().boxed()),
            ),
            ServerMethod::new(
                string_string_method("/test/Error"),
                MethodHandlerFn::new(|_| Err(GrpcError::Other("my error")).into_future().boxed()),
            ),
            ServerMethod::new(
                string_string_method("/test/Panic"),
                MethodHandlerFn::new(|_| panic!("icnap")),
            ),
        ]),
    )
}

struct TestClient {
    grpc_client: GrpcClient,
}

impl TestClient {
    fn new(port: u16) -> TestClient {
        TestClient {
            grpc_client: GrpcClient::new("::", port),
        }
    }

    fn call(&self, name: &str, param: &str) -> GrpcFuture<String> {
        self.grpc_client.call(param.to_owned(), string_string_method(name))
    }

    fn call_expect_error<F : FnOnce(&GrpcError) -> bool>(&self, name: &str, param: &str, expect: F) {
        match self.call(name, param).wait() {
            Ok(r) => panic!("expecting error, got: {:?}", r),
            Err(e) => assert!(expect(&e), "wrong error: {:?}", e),
        }
    }

    fn call_expect_grpc_error<F : FnOnce(&str) -> bool>(&self, name: &str, param: &str, expect: F) {
        self.call_expect_error(name, param, |e| {
            match e {
                &GrpcError::GrpcHttp(GrpcHttpError { ref grpc_message }) if expect(&grpc_message) => true,
                _ => false,
            }
        });
    }

    fn call_expect_grpc_error_contain(&self, name: &str, param: &str, expect: &str) {
        self.call_expect_grpc_error(name, param, |m| m.find(expect).is_some());
    }
}

fn client_and_server() -> (TestClient, GrpcServer) {
    let server = test_server();

    let client = TestClient::new(server.local_addr().port());

    (client, server)
}

#[test]
fn echo() {
    let (client, _server) = client_and_server();

    assert_eq!("aa", client.call("/test/Echo", "aa").wait().unwrap());
}

#[test]
fn error_in_handler() {
    let (client, _server) = client_and_server();

    client.call_expect_grpc_error_contain("/test/Error", "aa", "my error");
}

#[test]
fn panic_in_handler() {
    let (client, _server) = client_and_server();

    client.call_expect_grpc_error("/test/Panic", "aa",
        |m| m.find("panic").is_some() /* && m.find("icnap").is_some() */);
}
