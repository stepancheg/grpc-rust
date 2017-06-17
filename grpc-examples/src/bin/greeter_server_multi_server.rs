extern crate futures;

extern crate grpc_examples;
extern crate grpc;

use std::thread;

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
    let mut conf = grpc::ServerConf::default();
    conf.http.reuse_port = Some(true);

    let _server1 = GreeterServer::new("[::]:50051", conf.clone(), GreeterImpl)
        .expect("server 1");
    let _server2 = GreeterServer::new("[::]:50051", conf, GreeterImpl)
        .expect("server 2");

    loop {
        thread::park();
    }
}
