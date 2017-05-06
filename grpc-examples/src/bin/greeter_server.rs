extern crate grpc_examples;
extern crate grpc;

use std::thread;

use grpc::*;
use grpc::result::GrpcResult;

use grpc_examples::helloworld_grpc::*;
use grpc_examples::helloworld::*;

struct GreeterImpl;

impl Greeter for GreeterImpl {
    fn SayHello(&self, _m: GrpcRequestOptions, req: HelloRequest) -> GrpcResult<HelloReply> {
        let mut r = HelloReply::new();
        let name = if req.get_name().is_empty() { "world" } else { req.get_name() };
        println!("greeting request from {}", name);
        r.set_message(format!("Hello {}", name));
        Ok(r)
    }
}

fn main() {
    let _server = GreeterServer::new_plain("[::]:50051", Default::default(), GreeterImpl);

    loop {
        thread::park();
    }
}
