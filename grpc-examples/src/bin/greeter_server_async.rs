extern crate futures;

extern crate grpc_examples;
extern crate grpc;

use std::thread;

use futures::Future;

use grpc::futures_grpc::*;

use grpc_examples::helloworld_grpc::*;
use grpc_examples::helloworld::*;

struct GreeterImpl;

impl GreeterAsync for GreeterImpl {
    fn SayHello(&self, req: HelloRequest) -> GrpcFutureSend<HelloReply> {
        let mut r = HelloReply::new();
        let name = if req.get_name().is_empty() { "world" } else { req.get_name() };
        println!("greeting request from {}", name);
        r.set_message(format!("Hello {}", name));
        Box::new(futures::finished(r))
    }
}

fn main() {
    let _server = GreeterAsyncServer::new("localhost:50051", GreeterImpl);

    loop {
        thread::park();
    }
}
