#[macro_use]
extern crate log;

mod test_misc;

use grpc::rt::*;
use grpc::*;

use futures::executor;
use test_misc::*;

fn echo_fn(
    _: ServerHandlerContext,
    req: ServerRequestSingle<String>,
    resp: ServerResponseUnarySink<String>,
) -> grpc::Result<()> {
    resp.finish(req.message)
}

fn reverse_fn(
    _: ServerHandlerContext,
    req: ServerRequestSingle<String>,
    resp: ServerResponseUnarySink<String>,
) -> grpc::Result<()> {
    resp.finish(req.message.chars().rev().collect())
}

#[test]
fn multiple_services() {
    init_logger();

    let mut server = ServerBuilder::new_plain();
    server.http.set_port(0);

    let echo = string_string_method("/foo/echo", GrpcStreaming::Unary);
    let reverse = string_string_method("/bar/reverse", GrpcStreaming::Unary);

    server.add_service(ServerServiceDefinition::new(
        "/foo",
        vec![ServerMethod::new(
            echo.clone(),
            MethodHandlerUnary::new(echo_fn),
        )],
    ));

    server.add_service(ServerServiceDefinition::new(
        "/bar",
        vec![ServerMethod::new(
            reverse.clone(),
            MethodHandlerUnary::new(reverse_fn),
        )],
    ));

    let server = server.build().expect("server");

    let port = server.local_addr().port().expect("port");

    let client = ClientBuilder::new(BIND_HOST, port).build().expect("client");

    assert_eq!(
        "abc".to_owned(),
        executor::block_on(
            client
                .call_unary(RequestOptions::new(), "abc".to_owned(), echo)
                .drop_metadata()
        )
        .unwrap()
    );

    assert_eq!(
        "zyx".to_owned(),
        executor::block_on(
            client
                .call_unary(RequestOptions::new(), "xyz".to_owned(), reverse)
                .drop_metadata()
        )
        .unwrap()
    );
}

#[cfg(unix)]
#[test]
fn single_service_unix() {
    init_logger();

    let test_socket_address = "/tmp/grpc_rust_single_service_unix";
    let mut server = ServerBuilder::new_plain();
    server
        .http
        .set_unix_addr(test_socket_address.to_owned())
        .unwrap();

    let echo = string_string_method("/foo/echo", GrpcStreaming::Unary);
    let reverse = string_string_method("/bar/reverse", GrpcStreaming::Unary);

    server.add_service(ServerServiceDefinition::new(
        "/foo",
        vec![ServerMethod::new(
            echo.clone(),
            MethodHandlerUnary::new(echo_fn),
        )],
    ));

    server.add_service(ServerServiceDefinition::new(
        "/bar",
        vec![ServerMethod::new(
            reverse.clone(),
            MethodHandlerUnary::new(reverse_fn),
        )],
    ));

    let _server = server.build().expect("server");

    let client = ClientBuilder::new_unix(test_socket_address)
        .build()
        .expect("client");

    assert_eq!(
        "abc".to_owned(),
        executor::block_on(
            client
                .call_unary(RequestOptions::new(), "abc".to_owned(), echo)
                .drop_metadata()
        )
        .unwrap()
    );

    assert_eq!(
        "zyx".to_owned(),
        executor::block_on(
            client
                .call_unary(RequestOptions::new(), "xyz".to_owned(), reverse)
                .drop_metadata()
        )
        .unwrap()
    );
}
