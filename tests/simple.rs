extern crate grpc;
extern crate futures;

use std::sync::Arc;

use grpc::server::*;
use grpc::client::*;
use grpc::method::*;
use grpc::marshall::*;
use futures::*;

#[test]
fn test() {
    let echo = Arc::new(MethodDescriptor {
        name: "/test/Echo".to_owned(),
        client_streaming: false,
        server_streaming: false,
        req_marshaller: Box::new(MarshallerString),
        resp_marshaller: Box::new(MarshallerString),
    });

    let server = GrpcServer::new(
        0,
        ServerServiceDefinition::new(vec![
            ServerMethod::new(
                echo.clone(),
                MethodHandlerFn::new(|s| Ok(s).into_future().boxed()),
            ),
        ]),
    );

    let client = GrpcClient::new("::1", server.local_addr().port());

    assert_eq!("aa", client.call("aa".to_owned(), echo).wait().unwrap());
}
