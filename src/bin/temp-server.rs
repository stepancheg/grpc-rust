extern crate grpc;

fn main() {
    let mut server = grpc::server::GrpcServer::new();
    server.run();
}
