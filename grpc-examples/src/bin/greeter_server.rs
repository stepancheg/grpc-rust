extern crate futures;
extern crate futures_cpupool;

extern crate grpc_examples;
extern crate grpc;

use std::thread;

use futures_cpupool::CpuPool;

use grpc_examples::helloworld_grpc::*;
use grpc_examples::helloworld::*;

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

fn main() {
    let _server = GreeterServer::new_pool(
        "[::]:50051", Default::default(), GreeterImpl, CpuPool::new(4));

    loop {
        thread::park();
    }
}
