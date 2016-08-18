extern crate grpc_examples;
extern crate grpc;

use grpc_examples::helloworld_grpc::*;
use grpc_examples::helloworld::*;

struct GreeterImpl;

impl Greeter for GreeterImpl {
    fn SayHello(&self, req: HelloRequest) -> HelloReply {
        let mut r = HelloReply::new();
        let name = if req.get_name().is_empty() { "world" } else { req.get_name() };
        r.set_message(format!("Hello {}", name));
        r
    }
}

fn main() {
    let mut server = GreeterServer::new(GreeterImpl);
    server.run();

}
