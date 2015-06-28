extern crate grpc;

use grpc::codegen;

fn main() {
    codegen::protoc_gen_grpc_rust_main();
}
