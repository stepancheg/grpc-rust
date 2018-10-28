extern crate protoc_rust_grpc;

fn main() {
    protoc_rust_grpc::run(protoc_rust_grpc::Args {
        out_dir: "src",
        includes: &[],
        input: &["route_guide.proto"],
        rust_protobuf: true,
        ..Default::default()
    }).expect("protoc-rust-grpc");
}
