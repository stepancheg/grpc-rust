grpc-rust
=========

[![Join the chat at https://gitter.im/grpc-rust/Lobby](https://badges.gitter.im/grpc-rust/Lobby.svg)](https://gitter.im/grpc-rust/Lobby?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

<!-- https://travis-ci.org/stepancheg/rust-protobuf.png -->
[![Build Status](https://img.shields.io/travis/stepancheg/grpc-rust.svg)](https://travis-ci.org/stepancheg/grpc-rust)
[![License](https://img.shields.io/crates/l/grpc.svg)](https://github.com/stepancheg/grpc-rust/blob/master/LICENSE.txt)

Rust implementation of [gRPC](http://www.grpc.io/) protocol, under development.

## Current status

Client and server without streaming can be done with rust-grpc,
see `grpc-examples/src/bin/greeter_{client,server}{,_async}.rs`. It can be tested
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

### TODO

* Tests
* Proper error handling
* Upload to crates
* Streaming
* Performance
