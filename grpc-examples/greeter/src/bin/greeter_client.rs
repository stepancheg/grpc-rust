use std::env;
use std::sync::Arc;

use grpc_examples_greeter::helloworld::*;
use grpc_examples_greeter::helloworld_grpc::*;

use grpc::ClientStub;
use grpc::ClientStubExt;

use futures::executor;
use tls_api::TlsConnector;
use tls_api::TlsConnectorBuilder;

fn test_tls_connector() -> tls_api_native_tls::TlsConnector {
    let root_ca = include_bytes!("../root-ca.der");
    let root_ca = tls_api::Certificate::from_der(root_ca.to_vec());

    let mut builder = tls_api_native_tls::TlsConnector::builder().unwrap();
    builder
        .add_root_certificate(root_ca)
        .expect("add_root_certificate");
    builder.build().unwrap()
}

fn is_tls() -> bool {
    env::args().any(|a| a == "--tls")
}

fn main() {
    env_logger::init();

    let tls = is_tls();

    let name = env::args()
        .filter(|a| a != "--tls")
        .nth(1)
        .map(|s| s.to_owned())
        .unwrap_or_else(|| "world".to_owned());

    let port = if !tls { 50051 } else { 50052 };

    let client_conf = Default::default();

    let client = if tls {
        // This is a bit complicated, because we need to explicitly pass root CA here
        // because server uses self-signed certificate.
        // TODO: simplify it
        let tls_option =
            httpbis::ClientTlsOption::Tls("foobar.com".to_owned(), Arc::new(test_tls_connector()));
        let grpc_client = Arc::new(
            grpc::ClientBuilder::new("::1", port)
                .explicit_tls(tls_option)
                .build()
                .unwrap(),
        );
        GreeterClient::with_client(grpc_client)
    } else {
        GreeterClient::new_plain("::1", port, client_conf).unwrap()
    };

    let mut req = HelloRequest::new();
    req.set_name(name);

    let resp = client
        .say_hello(grpc::RequestOptions::new(), req)
        .join_metadata_result();

    println!("{:?}", executor::block_on(resp));
}
