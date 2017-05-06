extern crate protobuf;
extern crate grpc;
extern crate futures;
extern crate futures_cpupool;
extern crate log;
extern crate env_logger;
extern crate chrono;
extern crate clap;

extern crate grpc_interop;
use grpc_interop::*;

use grpc::*;

use chrono::*;
use clap::{App, Arg};

fn empty_unary(client: TestServiceClient) {
    client.EmptyCall(GrpcRequestOptions::new(), Empty::new()).expect("failed to get EmptyUnary result");
    println!("{} EmptyUnary done", Local::now().to_rfc3339());
}

// TODO: file a bug on this failure.
fn large_unary(client: TestServiceClient) {
    let mut payload = Payload::new();
    payload.set_body(vec![0; 271828]);
    let mut request = SimpleRequest::new();
    request.set_payload(payload);
    request.set_response_size(314159);
    let response = client.UnaryCall(GrpcRequestOptions::new(), request).expect("expected full frame");
    assert!(response.get_payload().body.len() == 314159);
    println!("{} LargeUnary done", Local::now().to_rfc3339());
}

fn client_streaming(client: TestServiceClient) {
    let mut requests = Vec::new();
    for size in [27182, 8, 1828, 45904].iter() {
        let mut request = StreamingInputCallRequest::new();
        let mut payload = Payload::new();
        payload.set_body(vec![0; size.to_owned()]);
        request.set_payload(payload);
        requests.push(Ok(request));
    }

    let response = client.StreamingInputCall(GrpcRequestOptions::new(), Box::new(requests.into_iter()))
        .expect("expected response");
    assert!(response.aggregated_payload_size == 74922);
    println!("{} ClientStreaming done", Local::now().to_rfc3339());
}

// TODO: test fails with an assertion error, we never see a response from the client.
// fails with 'expected a response: Other("partial frame")'
fn server_streaming(client: TestServiceClient) {
    let mut req = StreamingOutputCallRequest::new();
    let mut params = Vec::new();
    for &size in [31415, 9, 2653, 58979].iter() {
        let mut rp = ResponseParameters::new();
        rp.set_size(size as i32);
        params.push(rp);
    }
    req.set_response_parameters(::protobuf::RepeatedField::from_vec(params));

    let response_stream = client.StreamingOutputCall(GrpcRequestOptions::new(), req);

    let mut response_sizes = Vec::new();

    {
        // this scope is to satisfy the borrow checker.
        let bar = response_stream.map(|response| {
            response_sizes.push(response.expect("expected a response").get_payload().body.len());
        });
        assert!(bar.count() == 4);
    }

    assert!(response_sizes.len() == 4);
    assert!(response_sizes[0] == 31415);
    assert!(response_sizes[1] == 9);
    assert!(response_sizes[2] == 2653);
    assert!(response_sizes[3] == 58979);
    println!("{} ServerStreaming done", Local::now().to_rfc3339());
}

// TODO: this test needs to interleave requests with responses and check along the way.
// Need to find a way to model that with the blocking API.
fn ping_pong(client: TestServiceClient) {
    let mut requests = Vec::new();
    for &size_body_len in [(31415, 27182), (9, 8), (2653, 1828), (58979, 45904)].iter() {
        let mut req = StreamingOutputCallRequest::new();
        let mut params = ResponseParameters::new();
        params.set_size(size_body_len.0);
        req.set_response_parameters(::protobuf::RepeatedField::from_vec(vec![params]));
        let mut payload = Payload::new();
        payload.set_body(vec![0;size_body_len.1]);
        req.set_payload(payload);
        requests.push(Ok(req));
    }
    let response = client.FullDuplexCall(GrpcRequestOptions::new(), Box::new(requests.into_iter()));
    let mut response_sizes = Vec::new();
    {
        // this scope is to satisfy the borrow checker.
        let bar = response.map(|result| {
            response_sizes.push(result.unwrap().get_payload().body.len());
        });
        assert!(bar.count() == 4);
    }

    println!("response_sizes.len is {}", response_sizes.len());
    assert!(response_sizes.len() == 4);
    assert!(response_sizes[0] == 31415);
    assert!(response_sizes[1] == 9);
    assert!(response_sizes[2] == 2653);
    assert!(response_sizes[3] == 58979);
    println!("{} PingPong done", Local::now().to_rfc3339());
}

fn empty_stream(client: TestServiceClient) {
    let response = client.FullDuplexCall(GrpcRequestOptions::new(), Box::new(Vec::new().into_iter()));
    assert!(response.count() == 0);
    println!("{} EmptyStream done", Local::now().to_rfc3339());
}

// The flags we use are defined in the gRPC Interopability doc
// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md
fn main() {
    env_logger::init().expect("env_logger::init");

    let options = App::new("gRPC interopability client")
        .version("0.1")
        .author("Steve Jenson <stevej@buoyant.io>")
        .about("Interoperability Test Client for grpc-rust")
        .arg(Arg::with_name("server_host")
            .long("server_host")
            .help("The server host to connect to. For example, \"localhost\" or \"127.0.0.1\"")
            .takes_value(true))
        .arg(Arg::with_name("server_host_override")
            .long("server_host_override")
            .help("The server host to claim to be connecting to, for use in TLS and HTTP/2 \
                   :authority header. If unspecified, the value of --server_host will be used")
            .takes_value(true))
        .arg(Arg::with_name("server_port")
            .long("server_port")
            .help("The server port to connect to. For example, \"8080\"")
            .takes_value(true))
        .arg(Arg::with_name("test_case")
            .long("test_case")
            .help("The name of the test case to execute. For example, \"empty_unary\"")
            .takes_value(true))
        .arg(Arg::with_name("use_tls") // boolean
            .long("use_tls")
            .help("Whether to use a plaintext or encrypted connection")
            .takes_value(true))
        .arg(Arg::with_name("use_test_ca")
            .long("use_test_ca")
            .help("Whether to replace platform root CAs with ca.pem as the CA root")
            .takes_value(true))
        .arg(Arg::with_name("default_service_account")
            .long("default_service_account")
            .help("Email of the GCE default service account.")
            .takes_value(true))
        .arg(Arg::with_name("oauth_scope")
            .long("oauth_scope")
            .help("OAuth scope. For example, \"https://www.googleapis.com/auth/xapi.zoo\"")
            .takes_value(true))
        .arg(Arg::with_name("service_account_key_file")
            .long("service_account_key_file")
            .help("The path to the service account JSON key file generated from GCE developer \
                   console.")
            .takes_value(true))
        .get_matches();

    let hostname = options.value_of("server_host").unwrap_or("localhost");
    let serverport = options.value_of("server_port").map(|s| s.parse().unwrap()).unwrap_or(DEFAULT_PORT);

    let client = TestServiceClient::new(hostname, serverport, false, Default::default())
        .expect("init");

    let testcase = options.value_of("test_case").unwrap_or("");
    match testcase.as_ref() {
        "empty_unary" => empty_unary(client),
        "cacheable_unary" => panic!("cacheable_unary not done yet"),
        "large_unary" => large_unary(client),
        "client_compressed_unary" => panic!("client_compressed_unary not done yet"),
        "server_compressed_unary" => panic!("server_compressed_unary not done yet"),
        "client_streaming" => client_streaming(client),
        "client_compressed_streaming" => panic!("client_compressed_streaming not done yet"),
        "server_streaming" => server_streaming(client),
        "server_compressed_streaming" => panic!("server_compressed_streaming not done yet"),
        "ping_pong" => ping_pong(client),
        "empty_stream" => empty_stream(client),
        "compute_engine_creds" => panic!("compute_engine_creds not done yet"),
        "jwt_token_creds" => panic!("jwt_token_creds not done yet"),
        "oauth2_auth_token" => panic!("oauth2_auth_token not done yet"),
        "per_rpc_creds" => panic!("per_rpc_creds not done yet"),
        "custom_metadata" => panic!("custom_metadata not done yet"),
        "status_code_and_message" => panic!("status_code_and_message not done yet"),
        "unimplemented_method" => panic!("unimplemented_method not done yet"),
        "unimplemented_service" => panic!("unimplemented_service not done yet"),
        "cancel_after_begin" => panic!("cancel_after_begin not done yet"),
        "cancel_after_first_response" => panic!("cancel_after_first_response not done yet"),
        "timeout_on_sleeping_server" => panic!("timeout_on_sleeping_server not done yet"),
        _ => panic!("no test_case specified"),
    }
}
