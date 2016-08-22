extern crate grpc;

use std::sync::Arc;

use grpc::method::ServerServiceDefinition;
use grpc::method::ServerMethod;
use grpc::method::MethodDescriptor;
use grpc::method::MethodHandlerEcho;
use grpc::marshall::MarshallerBytes;

fn main() {
    let mut server = grpc::server::GrpcServer::new(Arc::new(
        ServerServiceDefinition::new(vec![
            ServerMethod::new(
                MethodDescriptor {
                    name: "/helloworld.Greeter/SayHello".to_owned(),
                    client_streaming: false,
                    server_streaming: false,
                    req_marshaller: Box::new(MarshallerBytes),
                    resp_marshaller: Box::new(MarshallerBytes),
                },
                MethodHandlerEcho,
            ),
        ]),
    ));
    server.run();
}
