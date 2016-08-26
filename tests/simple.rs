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
}

#[test]
fn echo() {
    let server = test_server();

    let client = TestClient::new(server.local_addr().port());

    assert_eq!("aa", client.call("/test/Echo", "aa").wait().unwrap());
}

#[test]
fn error_in_handler() {
    let server = test_server();

    let client = TestClient::new(server.local_addr().port());

    assert!(client.call("/test/Error", "aa").wait().is_err());
}
