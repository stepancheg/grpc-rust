extern crate bytes;
extern crate chrono;
extern crate clap;
extern crate env_logger;
extern crate futures;
extern crate futures_cpupool;
extern crate grpc;
extern crate log;
extern crate protobuf;

use futures::future::Future;

extern crate grpc_interop;
use grpc_interop::*;

use bytes::Bytes;

use grpc::*;

use chrono::*;
use clap::{App, Arg};
use std::time::SystemTime;

fn empty_unary(client: TestServiceClient) {
    client
        .empty_call(grpc::RequestOptions::new(), Empty::new())
        .wait_drop_metadata()
        .expect("failed to get EmptyUnary result");
    println!("{} EmptyUnary done", Local::now().to_rfc3339());
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#cacheable_unary
fn cacheable_unary(client: TestServiceClient) {
    let mut request = SimpleRequest::new();
    request.set_payload({
        let mut payload = Payload::new();
        payload.set_body(format!("{}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos()).into_bytes());
        payload
    });
    let mut options = RequestOptions::new();
    options.metadata.add(MetadataKey::from("x-user-ip"), "1.2.3.4".into());
    options.cachable = true;
    let (_, r1, _) = client.cacheable_unary_call(options.clone(), request.clone())
        .wait().expect("call");
    let (_, r2, _) = client.cacheable_unary_call(options, request)
        .wait().expect("call");
    assert_eq!(r1.get_payload(), r2.get_payload());
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#large_unary
fn large_unary(client: TestServiceClient) {
    let mut payload = Payload::new();
    payload.set_body(vec![0; 271828]);
    let mut request = SimpleRequest::new();
    request.set_payload(payload);
    request.set_response_size(314159);
    let response = client
        .unary_call(grpc::RequestOptions::new(), request)
        .wait_drop_metadata()
        .expect("expected full frame");
    assert_eq!(314159, response.get_payload().body.len());
    println!("{} LargeUnary done", Local::now().to_rfc3339());
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#client_compressed_unary
fn client_compressed_unary(_client: TestServiceClient) {
    unimplemented!()
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#server_compressed_unary
fn server_compressed_unary(_client: TestServiceClient) {
    unimplemented!()
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#client_streaming
fn client_streaming(client: TestServiceClient) {

    let (mut req, resp) = client
        .streaming_input_call(
            grpc::RequestOptions::new(),
        ).wait().expect("expected response");

    for size in [27182, 8, 1828, 45904].iter() {
        let mut request = StreamingInputCallRequest::new();
        let mut payload = Payload::new();
        payload.set_body(vec![0; size.to_owned()]);
        request.set_payload(payload);
        req.block_wait().expect("block_wait");
        req.send_data(request).expect("send_data");
    }

    req.finish().expect("finish");

    let resp = resp.wait_drop_metadata().expect("wait_drop_metadata");

    assert_eq!(resp.aggregated_payload_size, 74922);
    println!("{} ClientStreaming done", Local::now().to_rfc3339());
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#client_streaming
fn client_compressed_streaming(_client: TestServiceClient) {
    unimplemented!()
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#server_streaming
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

    let response_stream = client
        .streaming_output_call(grpc::RequestOptions::new(), req)
        .wait_drop_metadata();

    let mut response_sizes = Vec::new();

    {
        // this scope is to satisfy the borrow checker.
        let bar = response_stream.map(|response| {
            response_sizes.push(
                response
                    .expect("expected a response")
                    .get_payload()
                    .body
                    .len(),
            );
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

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#server_streaming
fn server_compressed_streaming(_client: TestServiceClient) {
    unimplemented!()
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#ping_pong
// TODO: this test needs to interleave requests with responses and check along the way.
// Need to find a way to model that with the blocking API.
fn ping_pong(client: TestServiceClient) {
    let (mut req, resp) = client
        .full_duplex_call(
            grpc::RequestOptions::new(),
        ).wait().expect("start request");

    let mut resp = resp.wait_drop_metadata();

    for &(size, body_len) in [(31415, 27182), (9, 8), (2653, 1828), (58979, 45904)].iter() {
        let mut req_m = StreamingOutputCallRequest::new();
        let mut params = ResponseParameters::new();
        params.set_size(size);
        req_m.set_response_parameters(::protobuf::RepeatedField::from_vec(vec![params]));
        let mut payload = Payload::new();
        payload.set_body(vec![0; body_len]);
        req_m.set_payload(payload);
        req.block_wait().expect("block_wait");
        req.send_data(req_m).expect("send_data");

        let resp_m = resp.next().expect("next").unwrap();
        assert_eq!(size as usize, resp_m.payload.get_ref().body.len());
    }

    req.finish().expect("finish");
    assert!(resp.next().is_none());

    println!("{} PingPong done", Local::now().to_rfc3339());
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#empty_stream
fn empty_stream(client: TestServiceClient) {
    let (mut req, resp) = client
        .full_duplex_call(grpc::RequestOptions::new()).wait().expect("wait");
    req.finish().expect("finish");
    let resp: Vec<_> = resp.wait_drop_metadata().collect();
    assert!(resp.len() == 0);
    println!("{} EmptyStream done", Local::now().to_rfc3339());
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#compute_engine_creds
fn compute_engine_creds(_client: TestServiceClient) {
    unimplemented!()
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#jwt_token_creds
fn jwt_token_creds(_client: TestServiceClient) {
    unimplemented!()
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#oauth2_auth_token
fn oauth2_auth_token(_client: TestServiceClient) {
    unimplemented!()
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#per_rpc_creds
fn per_rpc_creds(_client: TestServiceClient) {
    unimplemented!()
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#google_default_credentials
fn google_default_credentials(_client: TestServiceClient) {
    unimplemented!()
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#custom_metadata
fn custom_metadata(client: TestServiceClient) {
    fn make_options() -> grpc::RequestOptions {
        // The client attaches custom metadata with the following keys and values:
        // key: "x-grpc-test-echo-initial", value: "test_initial_metadata_value"
        // key: "x-grpc-test-echo-initial", value: "test_initial_metadata_value"
        let mut options = grpc::RequestOptions::new();
        options.metadata.add(
            MetadataKey::from("x-grpc-test-echo-initial"),
            Bytes::from("test_initial_metadata_value"),
        );
        options.metadata.add(
            MetadataKey::from("x-grpc-test-echo-trailing-bin"),
            Bytes::from(&b"\xab\xab\xab"[..]),
        );
        options
    }

    fn assert_result_metadata(initial: grpc::Metadata, trailing: grpc::Metadata) {
        assert_eq!(
            Some(&b"test_initial_metadata_value"[..]),
            initial.get("x-grpc-test-echo-initial")
        );
        assert_eq!(
            Some(&b"\xab\xab\xab"[..]),
            trailing.get("x-grpc-test-echo-trailing-bin")
        );
    }

    {
        // to a UnaryCall with request:
        // {
        //   response_size: 314159
        //   payload:{
        //     body: 271828 bytes of zeros
        //   }
        // }
        let mut req = SimpleRequest::new();
        req.set_response_size(314159);
        let mut payload = Payload::new();
        payload.set_body(vec![0; 271828]);
        req.set_payload(payload);
        let (initial, _result, trailing) = client
            .unary_call(make_options(), req)
            .wait()
            .expect("UnaryCall");

        assert_result_metadata(initial, trailing);
    }

    {
        // to a FullDuplexCall with request:
        // {
        //   response_parameters:{
        //     size: 314159
        //   }
        //   payload:{
        //     body: 271828 bytes of zeros
        //   }
        // }
        let mut req = StreamingOutputCallRequest::new();
        {
            let mut rp = ResponseParameters::new();
            rp.set_size(314159);
            req.mut_response_parameters().push(rp);
        }
        {
            let mut p = Payload::new();
            p.set_body(vec![0; 271828]);
            req.set_payload(p);
        }

        let (mut req1, resp) = client
            .full_duplex_call(make_options()).wait().expect("start request");
        req1.send_data(req).expect("send_data");
        req1.finish().expect("finish");

        let (initial, _, trailing) = resp.collect().wait().expect("collect");

        assert_result_metadata(initial, trailing);
    }
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#status_code_and_message
fn status_code_and_message(_client: TestServiceClient) {
    unimplemented!()
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#special_status_message
fn special_status_message(_client: TestServiceClient) {
    unimplemented!()
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#unimplemented_method
fn unimplemented_method(_client: TestServiceClient) {
    unimplemented!()
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#unimplemented_service
fn unimplemented_service(_client: TestServiceClient) {
    unimplemented!()
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#cancel_after_begin
fn cancel_after_begin(_client: TestServiceClient) {
    unimplemented!()
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#cancel_after_first_response
fn cancel_after_first_response(_client: TestServiceClient) {
    unimplemented!()
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#timeout_on_sleeping_server
fn timeout_on_sleeping_server(_client: TestServiceClient) {
    unimplemented!()
}

// The flags we use are defined in the gRPC Interopability doc
// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md
fn main() {
    env_logger::init();

    let options = App::new("gRPC interopability client")
        .version("0.1")
        .author("Steve Jenson <stevej@buoyant.io>")
        .about("Interoperability Test Client for grpc-rust")
        .arg(
            Arg::with_name("server_host")
                .long("server_host")
                .help("The server host to connect to. For example, \"localhost\" or \"127.0.0.1\"")
                .takes_value(true),
        ).arg(
            Arg::with_name("server_host_override")
                .long("server_host_override")
                .help(
                    "The server host to claim to be connecting to, for use in TLS and HTTP/2 \
                     :authority header. If unspecified, the value of --server_host will be used",
                ).takes_value(true),
        ).arg(
            Arg::with_name("server_port")
                .long("server_port")
                .help("The server port to connect to. For example, \"8080\"")
                .takes_value(true),
        ).arg(
            Arg::with_name("test_case")
                .long("test_case")
                .help("The name of the test case to execute. For example, \"empty_unary\"")
                .takes_value(true),
        ).arg(
            Arg::with_name("use_tls") // boolean
                .long("use_tls")
                .help("Whether to use a plaintext or encrypted connection")
                .takes_value(true),
        ).arg(
            Arg::with_name("use_test_ca")
                .long("use_test_ca")
                .help("Whether to replace platform root CAs with ca.pem as the CA root")
                .takes_value(true),
        ).arg(
            Arg::with_name("default_service_account")
                .long("default_service_account")
                .help("Email of the GCE default service account.")
                .takes_value(true),
        ).arg(
            Arg::with_name("oauth_scope")
                .long("oauth_scope")
                .help("OAuth scope. For example, \"https://www.googleapis.com/auth/xapi.zoo\"")
                .takes_value(true),
        ).arg(
            Arg::with_name("service_account_key_file")
                .long("service_account_key_file")
                .help(
                    "The path to the service account JSON key file generated from GCE developer \
                     console.",
                ).takes_value(true),
        ).get_matches();

    let hostname = options.value_of("server_host").unwrap_or("127.0.0.1");
    let serverport = options
        .value_of("server_port")
        .map(|s| s.parse().unwrap())
        .unwrap_or(DEFAULT_PORT);

    let client =
        TestServiceClient::new_plain(hostname, serverport, Default::default()).expect("init");

    let testcase = options.value_of("test_case").unwrap_or("");
    match testcase.as_ref() {
        "empty_unary" => empty_unary(client),
        "cacheable_unary" => cacheable_unary(client),
        "large_unary" => large_unary(client),
        "client_compressed_unary" => client_compressed_unary(client),
        "server_compressed_unary" => server_compressed_unary(client),
        "client_streaming" => client_streaming(client),
        "client_compressed_streaming" => client_compressed_streaming(client),
        "server_streaming" => server_streaming(client),
        "server_compressed_streaming" => server_compressed_streaming(client),
        "ping_pong" => ping_pong(client),
        "empty_stream" => empty_stream(client),
        "compute_engine_creds" => compute_engine_creds(client),
        "jwt_token_creds" => jwt_token_creds(client),
        "oauth2_auth_token" => oauth2_auth_token(client),
        "per_rpc_creds" => per_rpc_creds(client),
        "google_default_credentials" => google_default_credentials(client),
        "custom_metadata" => custom_metadata(client),
        "status_code_and_message" => status_code_and_message(client),
        "special_status_message" => special_status_message(client),
        "unimplemented_method" => unimplemented_method(client),
        "unimplemented_service" => unimplemented_service(client),
        "cancel_after_begin" => cancel_after_begin(client),
        "cancel_after_first_response" => cancel_after_first_response(client),
        "timeout_on_sleeping_server" => timeout_on_sleeping_server(client),
        _ => panic!("no test_case specified"),
    }
}
