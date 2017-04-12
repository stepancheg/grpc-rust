extern crate grpc_examples;
extern crate grpc;
extern crate futures;

use grpc::*;

use grpc_examples::helloworld_grpc::*;
use grpc_examples::helloworld::*;

use std::env;


fn main() {
    let name = env::args().nth(1).map(|s| s.to_owned()).unwrap_or_else(|| "world".to_owned());

    let client = GreeterClient::new("localhost", 50051, false, Default::default()).unwrap();

    let mut req = HelloRequest::new();
    req.set_name(name);

    let resp = client.say_hello(GrpcRequestOptions::new(), req);

    println!("{:?}", resp.wait());
}
