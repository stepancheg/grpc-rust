# grpc compiler

Stub generator for grpc.

Library and `protoc-gen-rust-grpc` plugin for `protoc`.

Can be installed with command `cargo install grpc-compiler`.

Generate .rs files:

```
protoc --rust-grpc_out=. foo.proto
```

Generate .rs files (protobuf messages and grpc stubs):

```
protoc --rust_out=. --rust-grpc_out=. foo.proto
```

Alternatively, [protoc-grpc-rust](https://github.com/stepancheg/grpc-rust/tree/master/protoc-rust-grpc)
crate can be used to invoke codegen programmatically, which only requires `protoc` command in `$PATH`.
