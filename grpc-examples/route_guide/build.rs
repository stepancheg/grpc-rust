fn main() {
    protoc_rust_grpc::Codegen::new()
        .out_dir("src")
        .input("route_guide.proto")
        .rust_protobuf(true)
        .run()
        .expect("protoc-rust-grpc");
}
