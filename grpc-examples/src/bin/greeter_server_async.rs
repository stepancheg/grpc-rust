extern crate futures;

extern crate grpc_examples;
extern crate grpc;

use std::thread;

use grpc::*;

use grpc_examples::helloworld_grpc::*;
use grpc_examples::helloworld::*;

struct GreeterImpl;

impl GreeterAsync for GreeterImpl {
    fn SayHello(&self, _m: GrpcRequestOptions, req: HelloRequest) -> GrpcSingleResponse<HelloReply> {
        let mut r = HelloReply::new();
        let name = if req.get_name().is_empty() { "world" } else { req.get_name() };
        println!("greeting request from {}", name);
        r.set_message(format!("Hello {}", name));
        GrpcSingleResponse::completed(r)
    }
}

fn main() {
    let _server = GreeterAsyncServer::new("[::]:50051", Default::default(), GreeterImpl);

    loop {
        thread::park();
    }
}
