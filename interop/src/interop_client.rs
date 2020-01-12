use bytes::Bytes;

use grpc::*;

use crate::empty::Empty;
use crate::messages::Payload;
use crate::messages::ResponseParameters;
use crate::messages::SimpleRequest;
use crate::messages::StreamingInputCallRequest;
use crate::messages::StreamingOutputCallRequest;
use crate::test_grpc::TestServiceClient;
use chrono::*;
use futures::executor;
use futures::future;
use futures::stream::StreamExt;
use std::time::SystemTime;

fn empty_unary(client: TestServiceClient) {
    executor::block_on(
        client
            .empty_call(grpc::RequestOptions::new(), Empty::new())
            .drop_metadata(),
    )
    .expect("failed to get EmptyUnary result");
    println!("{} EmptyUnary done", Local::now().to_rfc3339());
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#cacheable_unary
fn cacheable_unary(client: TestServiceClient) {
    let mut request = SimpleRequest::new();
    request.set_payload({
        let mut payload = Payload::new();
        payload.set_body(
            format!(
                "{}",
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            )
            .into_bytes(),
        );
        payload
    });
    let mut options = RequestOptions::new();
    options
        .metadata
        .add(MetadataKey::from("x-user-ip"), "1.2.3.4".into());
    options.cachable = true;
    let r1 = executor::block_on(
        client
            .cacheable_unary_call(options.clone(), request.clone())
            .drop_metadata(),
    )
    .expect("call");
    let r2 = executor::block_on(
        client
            .cacheable_unary_call(options, request)
            .drop_metadata(),
    )
    .expect("call");
    assert_eq!(r1.get_payload(), r2.get_payload());
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#large_unary
fn large_unary(client: TestServiceClient) {
    let mut payload = Payload::new();
    payload.set_body(vec![0; 271828]);
    let mut request = SimpleRequest::new();
    request.set_payload(payload);
    request.set_response_size(314159);
    let response = executor::block_on(
        client
            .unary_call(grpc::RequestOptions::new(), request)
            .drop_metadata(),
    )
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
    let (mut req, resp) =
        executor::block_on(client.streaming_input_call(grpc::RequestOptions::new()))
            .expect("expected response");

    for size in [27182, 8, 1828, 45904].iter() {
        let mut request = StreamingInputCallRequest::new();
        let mut payload = Payload::new();
        payload.set_body(vec![0; size.to_owned()]);
        request.set_payload(payload);
        executor::block_on(future::poll_fn(|cx| req.poll(cx))).expect("block_wait");
        req.send_data(request).expect("send_data");
    }

    req.finish().expect("finish");

    let resp = executor::block_on(resp.drop_metadata()).expect("wait_drop_metadata");

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

    let mut response_stream = client
        .streaming_output_call(grpc::RequestOptions::new(), req)
        .drop_metadata();

    let mut response_sizes = Vec::new();

    {
        let bar = executor::block_on(async {
            let mut count = 0;
            while let Some(response) = response_stream
                .next()
                .await
                .transpose()
                .expect("expected as response")
            {
                response_sizes.push(response.get_payload().body.len());
                count += 1;
            }
            count
        });
        assert!(bar == 4);
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
fn ping_pong(client: TestServiceClient) {
    let (mut req, resp) = executor::block_on(client.full_duplex_call(grpc::RequestOptions::new()))
        .expect("start request");

    let mut resp = resp.drop_metadata();

    for &(size, body_len) in [(31415, 27182), (9, 8), (2653, 1828), (58979, 45904)].iter() {
        let mut req_m = StreamingOutputCallRequest::new();
        let mut params = ResponseParameters::new();
        params.set_size(size);
        req_m.set_response_parameters(::protobuf::RepeatedField::from_vec(vec![params]));
        let mut payload = Payload::new();
        payload.set_body(vec![0; body_len]);
        req_m.set_payload(payload);
        executor::block_on(req.wait()).expect("block_wait");
        req.send_data(req_m).expect("send_data");

        let resp_m = executor::block_on(resp.next()).expect("next").unwrap();
        assert_eq!(size as usize, resp_m.payload.get_ref().body.len());
    }

    req.finish().expect("finish");
    assert!(executor::block_on(resp.next()).is_none());

    println!("{} PingPong done", Local::now().to_rfc3339());
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#empty_stream
fn empty_stream(client: TestServiceClient) {
    let (mut req, resp) =
        executor::block_on(client.full_duplex_call(grpc::RequestOptions::new())).expect("wait");
    req.finish().expect("finish");
    let resp: Vec<_> = executor::block_on(resp.drop_metadata().collect());
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
        let (initial, _result, trailing) = executor::block_on(
            client
                .unary_call(make_options(), req)
                .join_metadata_result(),
        )
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

        let (mut req1, resp) =
            executor::block_on(client.full_duplex_call(make_options())).expect("start request");
        req1.send_data(req).expect("send_data");
        req1.finish().expect("finish");

        let (initial, _, trailing) = executor::block_on(resp.collect()).expect("collect");

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
fn cancel_after_begin(client: TestServiceClient) {
    let (req, resp) =
        executor::block_on(client.streaming_input_call(RequestOptions::new())).expect("start");
    drop(req);
    // TODO: hangs
    match executor::block_on(resp) {
        Ok(_) => panic!("expecting err"),
        Err(_) => unimplemented!(),
    }
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#cancel_after_first_response
fn cancel_after_first_response(_client: TestServiceClient) {
    unimplemented!()
}

// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md#timeout_on_sleeping_server
fn timeout_on_sleeping_server(_client: TestServiceClient) {
    unimplemented!()
}

pub static TESTS: &[(&str, fn(TestServiceClient))] = &[
    ("empty_unary", empty_unary),
    ("cacheable_unary", cacheable_unary),
    ("large_unary", large_unary),
    ("client_compressed_unary", client_compressed_unary),
    ("server_compressed_unary", server_compressed_unary),
    ("client_streaming", client_streaming),
    ("client_compressed_streaming", client_compressed_streaming),
    ("server_streaming", server_streaming),
    ("server_compressed_streaming", server_compressed_streaming),
    ("ping_pong", ping_pong),
    ("empty_stream", empty_stream),
    ("compute_engine_creds", compute_engine_creds),
    ("jwt_token_creds", jwt_token_creds),
    ("oauth2_auth_token", oauth2_auth_token),
    ("per_rpc_creds", per_rpc_creds),
    ("google_default_credentials", google_default_credentials),
    ("custom_metadata", custom_metadata),
    ("status_code_and_message", status_code_and_message),
    ("special_status_message", special_status_message),
    ("unimplemented_method", unimplemented_method),
    ("unimplemented_service", unimplemented_service),
    ("cancel_after_begin", cancel_after_begin),
    ("cancel_after_first_response", cancel_after_first_response),
    ("timeout_on_sleeping_server", timeout_on_sleeping_server),
];
