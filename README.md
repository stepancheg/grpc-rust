grpc-rust
=========

<!-- https://travis-ci.org/stepancheg/rust-protobuf.png -->
[![Build Status](https://img.shields.io/travis/stepancheg/grpc-rust.svg)](https://travis-ci.org/stepancheg/grpc-rust)
[![License](https://img.shields.io/crates/l/grpc.svg)](https://github.com/stepancheg/grpc-rust/blob/master/LICENSE.txt)

Rust implementation of [gRPC](http://www.grpc.io/) protocol, under development.

Some development questions in [FAQ](/docs/FAQ.md).

## Current status

It basially works. See `grpc-examples/src/bin/greeter_{client,server}{,_async}.rs`. It can be tested
for example with [go client](https://github.com/grpc/grpc-go/tree/master/examples/helloworld):

```
# start greeter server implemented in rust
$ cargo run --bin greeter_server

# .. or start async server
$ cargo run --bin greeter_server_async

# ... or start greeter server implemented in go
$ go get -u google.golang.org/grpc/examples/helloworld/greeter_client
$ greeter_server

# start greeter client implemented in rust
$ cargo run --bin greeter_client rust
> message: "Hello rust"

# ... or start async client
$ cargo run --bin greeter_client_async rust
> message: "Hello rust"

# ... or start greeter client implemented in go
$ go get -u google.golang.org/grpc/examples/helloworld/greeter_client
$ greeter_client rust
> 2016/08/19 05:44:45 Greeting: Hello rust
```

Client and server are implemented asynchronously, and sync versions are thin wrappers around async counterparts.

## How to use gRPC compiler

### Build & install Rust protobuf and gRPC compiler:

```bash
cargo install protobuf
cargo install grpc-compiler
```

These commands install `protoc-gen-rust` and `protoc-gen-rust-grpc`
to `~/.cargo/bin`, which should be added to `$PATH`.

### Compile your proto & gRPC to Rust:

```bash
cd $YOURPROJECT
mkdir -p src
protoc --rust_out=src *.proto
protoc --rust-grpc_out=src *.proto
```

### Use compiled protos in your project:

In Cargo.toml:

```ini
[dependencies]
grpc            = "0.*"
protobuf        = "1.*"
futures         = "0.1"
futures-cpupool = "0.1"
```

In `lib.rs` or `main.rs` (or any other submodule):

```rust
extern crate protobuf;
extern crate grpc;
extern crate futures;
extern crate futures_cpupool;

pub mod myproto;
pub mod myproto_grpc;
```

### Compiling protos manually is silly. Can Cargo do all of above for me?

It seems possible, but looks like it requires some more work.

See https://github.com/stepancheg/rust-protobuf/issues/57 and
https://github.com/dwrensha/capnpc-rust for more details.

## TODO

* Implement flow control. See issue #35
* Fix performance
* More tests
* In particular, add more compatibility tests, they live in `long-tests` directory
* Fix all TODO in sources
