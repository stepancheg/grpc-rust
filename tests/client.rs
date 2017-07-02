#[macro_use]
extern crate log;
extern crate futures;

extern crate grpc;

mod test_misc;

use grpc::*;
use grpc::rt::GrpcStreaming;

use test_misc::*;


#[test]
fn server_is_not_running() {
    let client = Client::new_plain("::1", 2, Default::default()).unwrap();

    // TODO: https://github.com/tokio-rs/tokio-core/issues/12
    if false {
        let result = client.call_unary(
            RequestOptions::new(),
            "aa".to_owned(),
            string_string_method("/does/not/matter", GrpcStreaming::Unary)).wait();
        assert!(result.is_err(), result);
    }
}
