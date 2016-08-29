extern crate grpc;
extern crate futures;

use std::sync::Arc;

use futures::*;
use futures::stream::Stream;

use grpc::server::*;
use grpc::client::*;
use grpc::method::*;
use grpc::marshall::*;
use grpc::error::*;
use grpc::futures_grpc::*;


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

fn test_server() -> GrpcServer {
    GrpcServer::new(
        0,
        ServerServiceDefinition::new(vec![
            ServerMethod::new(
                string_string_method("/test/Echo", false, false),
                MethodHandlerUnary::new(|s| Ok(s).into_future().boxed()),
            ),
            ServerMethod::new(
                string_string_method("/test/ServerStreaming", false, true),
                MethodHandlerServerStreaming::new(|s| {
                    stream::iter((0..3).map(move |i| Ok(format!("{}{}", s, i)))).boxed()
                }),
            ),
            ServerMethod::new(
                string_string_method("/test/Error", false, false),
                MethodHandlerUnary::new(|_| Err(GrpcError::Other("my error")).into_future().boxed()),
            ),
            ServerMethod::new(
                string_string_method("/test/Panic", false, false),
                MethodHandlerUnary::new(|_| panic!("icnap")),
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

fn client_and_server() -> (TestClient, GrpcServer) {
    let server = test_server();

    let client = TestClient::new(server.local_addr().port());

    (client, server)
}

#[test]
fn unary() {
    let (client, _server) = client_and_server();

    assert_eq!("aa", client.call_unary("/test/Echo", "aa").wait().unwrap());
}

#[test]
fn server_streaming() {
    let (client, _server) = client_and_server();

    let rs = client.call_server_streaming("/test/ServerStreaming", "x").collect().wait().unwrap();
    // TODO: test it is really streaming (it is actually not)
    let expected = vec!["x0", "x1", "x2"];

    assert_eq!(expected, rs);
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
