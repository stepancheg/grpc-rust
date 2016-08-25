extern crate grpc_examples;
extern crate grpc;

use std::thread;

use grpc::result::GrpcResult;

use grpc_examples::helloworld_grpc::*;
use grpc_examples::helloworld::*;

struct GreeterImpl;

impl Greeter for GreeterImpl {
    fn SayHello(&self, req: HelloRequest) -> GrpcResult<HelloReply> {
        let mut r = HelloReply::new();
        let name = if req.get_name().is_empty() { "world" } else { req.get_name() };
        println!("greeting request from {}", name);
        r.set_message(format!("Hello {}", name));
        Ok(r)
    }
}

fn main() {
    let _server = GreeterServer::new(50051, GreeterImpl);

    loop {
        thread::park();
    }
}
