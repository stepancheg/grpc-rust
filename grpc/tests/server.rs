#[macro_use]
extern crate log;

extern crate futures;
extern crate grpc;

mod test_misc;

use grpc::*;
use grpc::rt::*;

use test_misc::*;


fn echo_fn(_: RequestOptions, req: String) -> SingleResponse<String> {
    SingleResponse::completed(req)
}

fn reverse_fn(_: RequestOptions, req: String) -> SingleResponse<String> {
    SingleResponse::completed(req.chars().rev().collect())
}


#[test]
fn multiple_services() {
    let mut server = ServerBuilder::new_plain();
    server.http.set_port(0);

    let echo = string_string_method("/foo/echo", GrpcStreaming::Unary);
    let reverse = string_string_method("/bar/reverse", GrpcStreaming::Unary);

    server.add_service(ServerServiceDefinition::new("/foo", vec![
        ServerMethod::new(
            echo.clone(),
            MethodHandlerUnary::new(echo_fn))
    ]));

    server.add_service(ServerServiceDefinition::new("/bar", vec![
        ServerMethod::new(
            reverse.clone(),
            MethodHandlerUnary::new(reverse_fn))
    ]));

    let server = server.build().expect("server");

    let client = Client::new_plain("::1", server.local_addr().port().unwrap(), ClientConf::new())
        .expect("client");

    assert_eq!(
        "abc".to_owned(),
        client.call_unary(
            RequestOptions::new(), "abc".to_owned(), echo).wait_drop_metadata().unwrap());

    assert_eq!(
        "zyx".to_owned(),
        client.call_unary(
            RequestOptions::new(), "xyz".to_owned(), reverse).wait_drop_metadata().unwrap());
}

#[cfg(unix)]
#[test]
fn single_service_unix() {
    let test_socket_address = "/tmp/grpc_rust_single_service_unix";
    let mut server = ServerBuilder::new_plain();
    server.http.set_unix_addr(test_socket_address.to_owned()).unwrap();

    let echo = string_string_method("/foo/echo", GrpcStreaming::Unary);
    let reverse = string_string_method("/bar/reverse", GrpcStreaming::Unary);

    server.add_service(ServerServiceDefinition::new("/foo", vec![
        ServerMethod::new(
            echo.clone(),
            MethodHandlerUnary::new(echo_fn))
    ]));

    server.add_service(ServerServiceDefinition::new("/bar", vec![
        ServerMethod::new(
            reverse.clone(),
            MethodHandlerUnary::new(reverse_fn))
    ]));

    let _server = server.build().expect("server");

    let client = Client::new_plain_unix(test_socket_address, ClientConf::new())
        .expect("client");

    assert_eq!(
        "abc".to_owned(),
        client.call_unary(
            RequestOptions::new(), "abc".to_owned(), echo).wait_drop_metadata().unwrap());

    assert_eq!(
        "zyx".to_owned(),
        client.call_unary(
            RequestOptions::new(), "xyz".to_owned(), reverse).wait_drop_metadata().unwrap());

}
