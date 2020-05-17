use grpc::prelude::*;

use futures::executor;
use grpc::ClientConf;
use grpc_examples_route_guide::client::run_client;
use grpc_examples_route_guide::route_guide_grpc::RouteGuideClient;
use grpc_examples_route_guide::DEFAULT_PORT;

fn main() {
    env_logger::init().unwrap();

    let client =
        RouteGuideClient::new_plain("127.0.0.1", DEFAULT_PORT, ClientConf::new()).expect("client");

    executor::block_on(async { run_client(&client).await });
}
