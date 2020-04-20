#![cfg(test)]

use crate::client::run_client;
use crate::route_guide_grpc::RouteGuideClient;
use crate::route_guide_grpc::RouteGuideServer;
use crate::server::RouteGuideImpl;
use futures::executor;
use grpc::ClientConf;
use grpc::ClientStubExt;

#[test]
fn test() {
    let service_def = RouteGuideServer::new_service_def(RouteGuideImpl::new_and_load_db());

    let mut server_builder = grpc::ServerBuilder::new_plain();
    server_builder.add_service(service_def);
    server_builder.http.set_port(0);
    let server = server_builder.build().expect("build");

    let port = server.local_addr().port().unwrap();

    // server is running now

    let client = RouteGuideClient::new_plain("127.0.0.1", port, ClientConf::new()).expect("client");

    executor::block_on(async { run_client(&client).await });

    // shutdown server
    drop(server);
}
