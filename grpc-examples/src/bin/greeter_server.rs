extern crate futures;

extern crate grpc_examples;
extern crate grpc;
extern crate tls_api;
extern crate tls_api_native_tls;

use std::thread;
use std::env;

use grpc_examples::helloworld_grpc::*;
use grpc_examples::helloworld::*;

use tls_api::TlsAcceptorBuilder;


struct GreeterImpl;

impl Greeter for GreeterImpl {
    fn say_hello(&self, _m: grpc::RequestOptions, req: HelloRequest) -> grpc::SingleResponse<HelloReply> {
        let mut r = HelloReply::new();
        let name = if req.get_name().is_empty() { "world" } else { req.get_name() };
        println!("greeting request from {}", name);
        r.set_message(format!("Hello {}", name));
        grpc::SingleResponse::completed(r)
    }
}

fn test_tls_acceptor() -> tls_api_native_tls::TlsAcceptor {
    let pkcs12 = include_bytes!("../foobar.com.p12");
    let builder = tls_api_native_tls::TlsAcceptorBuilder::from_pkcs12(pkcs12, "mypass").unwrap();
    builder.build().unwrap()
}

fn is_tls() -> bool {
    env::args().any(|a| a == "--tls")
}

fn main() {
    let tls = is_tls();

    let port = if !tls { 50051 } else { 50052 };
    
    let mut server = grpc::ServerBuilder::new();
    server.http.set_port(port);
    server.add_service(GreeterServer::new_service_def(GreeterImpl));
    server.http.set_cpu_pool_threads(4);
    if tls {
        server.http.set_tls(test_tls_acceptor());
    }
    let _server = server.build().expect("server");

    println!("greeter server started on port {} {}",
        port, if tls { "with tls" } else { "without tls" });

    loop {
        thread::park();
    }
}
