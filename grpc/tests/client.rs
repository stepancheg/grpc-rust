#[macro_use]
extern crate log;

mod test_misc;

use grpc::rt::GrpcStreaming;
use grpc::*;

use futures::executor;
use test_misc::*;

#[test]
fn server_is_not_running() {
    init_logger();

    let client = ClientBuilder::new(BIND_HOST, 2).build().unwrap();

    // TODO: https://github.com/tokio-rs/tokio-core/issues/12
    if false {
        let result = executor::block_on(
            client
                .call_unary(
                    RequestOptions::new(),
                    "aa".to_owned(),
                    string_string_method("/does/not/matter", GrpcStreaming::Unary),
                )
                .join_metadata_result(),
        );
        assert!(result.is_err(), result);
    }
}

#[cfg(unix)]
#[test]
fn server_is_not_running_unix() {
    init_logger();
    let client = ClientBuilder::new_unix("/tmp/grpc_rust_test")
        .build()
        .unwrap();

    // TODO: https://github.com/tokio-rs/tokio-core/issues/12
    if false {
        let result = executor::block_on(
            client
                .call_unary(
                    RequestOptions::new(),
                    "aa".to_owned(),
                    string_string_method("/does/not/matter", GrpcStreaming::Unary),
                )
                .join_metadata_result(),
        );
        assert!(result.is_err(), result);
    }
}
