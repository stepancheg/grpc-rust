extern crate futures;

extern crate grpc;
extern crate grpc_examples_greeter;
extern crate httpbis;

use std::thread;

use grpc::ServerHandlerContext;
use grpc::ServerRequestSingle;
use grpc::ServerResponseUnarySink;
use grpc_examples_greeter::helloworld::*;
use grpc_examples_greeter::helloworld_grpc::*;

struct GreeterImpl {
    instance: u32,
}

impl Greeter for GreeterImpl {
    fn say_hello(
        &self,
        _: ServerHandlerContext,
        req: ServerRequestSingle<HelloRequest>,
        resp: ServerResponseUnarySink<HelloReply>,
    ) -> grpc::Result<()> {
        let mut r = HelloReply::new();
        let name = if req.message.get_name().is_empty() {
            "world"
        } else {
            req.message.get_name()
        };
        println!("instance {}, greeting request from {}", self.instance, name);
        r.set_message(format!("Hello {}", name));
        resp.finish(r)
    }
}

fn main() {
    let mut conf = httpbis::ServerConf::default();
    conf.reuse_port = Some(true);

    let mut server1 = grpc::ServerBuilder::new_plain();
    server1.http.conf = conf.clone();
    server1.http.set_port(50051);
    server1.add_service(GreeterServer::new_service_def(GreeterImpl { instance: 1 }));
    let _server1 = server1.build().expect("server 1");

    let mut server2 = grpc::ServerBuilder::new_plain();
    server2.http.conf = conf.clone();
    server2.http.set_port(50051);
    server2.add_service(GreeterServer::new_service_def(GreeterImpl { instance: 2 }));
    let _server2 = server2.build().expect("server 1");

    loop {
        thread::park();
    }
}
