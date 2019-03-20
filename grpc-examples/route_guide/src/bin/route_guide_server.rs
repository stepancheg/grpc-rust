use grpc_examples_route_guide::route_guide_grpc::RouteGuideServer;
use grpc_examples_route_guide::server::RouteGuideImpl;
use grpc_examples_route_guide::DEFAULT_PORT;
use std::thread;

fn main() {
    let service_def = RouteGuideServer::new_service_def(RouteGuideImpl::new_and_load_db());

    let mut server_builder = grpc::ServerBuilder::new_plain();
    server_builder.add_service(service_def);
    server_builder.http.set_port(DEFAULT_PORT);
    let server = server_builder.build().expect("build");

    println!("server stared on addr {}", server.local_addr());

    loop {
        thread::park();
    }
}
