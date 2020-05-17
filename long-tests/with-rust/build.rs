fn main() {
    protoc_rust_grpc::Codegen::new()
        .out_dir("src")
        .include("..")
        .input("../long_tests_pb.proto")
        .rust_protobuf(true)
        .run()
        .expect("protoc-rust-grpc");
}
