extern crate futures;

extern crate grpc_examples;
extern crate grpc;

use std::thread;

use futures::Future;

use grpc::futures_grpc::GrpcFuture;

use grpc_examples::helloworld_grpc::*;
use grpc_examples::helloworld::*;

struct GreeterImpl;

impl GreeterAsync for GreeterImpl {
    fn SayHello(&self, req: HelloRequest) -> GrpcFuture<HelloReply> {
        let mut r = HelloReply::new();
        let name = if req.get_name().is_empty() { "world" } else { req.get_name() };
        println!("greeting request from {}", name);
        r.set_message(format!("Hello {}", name));
        futures::finished(r).boxed()
    }
}

fn main() {
    let _server = GreeterAsyncServer::new(50051, GreeterImpl);

    loop {
        thread::park();
    }
}
