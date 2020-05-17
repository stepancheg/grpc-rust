fn main() {
    protoc_rust_grpc::Codegen::new()
        .out_dir("src")
        .include("proto")
        .inputs(&[
            "proto/messages.proto",
            "proto/empty.proto",
            "proto/test.proto",
        ])
        .rust_protobuf(true)
        .run()
        .expect("protoc-rust-grpc");
}
