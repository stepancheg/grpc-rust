extern crate futures;

extern crate grpc_examples;
extern crate grpc;
extern crate httpbis;

use std::thread;

use grpc_examples::helloworld_grpc::*;
use grpc_examples::helloworld::*;

struct GreeterImpl {
    instance: u32,
}

impl Greeter for GreeterImpl {
    fn say_hello(&self, _m: grpc::RequestOptions, req: HelloRequest) -> grpc::SingleResponse<HelloReply> {
        let mut r = HelloReply::new();
        let name = if req.get_name().is_empty() { "world" } else { req.get_name() };
        println!("instance {}, greeting request from {}", self.instance, name);
        r.set_message(format!("Hello {}", name));
        grpc::SingleResponse::completed(r)
    }
}

fn main() {
    
    let mut conf = httpbis::ServerConf::default();
    conf.reuse_port = Some(true);

    let mut server1 = grpc::ServerBuilder::new_plain();
    server1.http.conf = conf.clone();
    server1.http.set_port(50051);
    server1.set_service(GreeterServer::new_service_def(GreeterImpl { instance: 1 }));
    let _server1 = server1.build().expect("server 1");

    let mut server2 = grpc::ServerBuilder::new_plain();
    server2.http.conf = conf.clone();
    server2.http.set_port(50051);
    server2.set_service(GreeterServer::new_service_def(GreeterImpl { instance: 2 }));
    let _server2 = server2.build().expect("server 1");

    loop {
        thread::park();
    }
}
