# grpc-rust

Rust implementation of [gRPC](http://www.grpc.io/) protocol, under development.

## Current status

Client and server without streaming can be done with rust-grpc,
see `grpc-examples/src/bin/greeter_{client,server}.rs`. It can be tested
for example with [go client](https://github.com/grpc/grpc-go/tree/master/examples/helloworld):

```
# start greeter server implemented in rust
$ cargo run --bin greeter_server

# ... or start greeter server implemented in go
$ go get -u google.golang.org/grpc/examples/helloworld/greeter_client
$ greeter_server

# start greeter client implemented in rust
$ cargo run --bin greeter_client rust
> message: "Hello rust"

# ... or start greeter client implemented in go
$ go get -u google.golang.org/grpc/examples/helloworld/greeter_client
$ greeter_client rust
> 2016/08/19 05:44:45 Greeting: Hello rust
```

Client is implemented asynchronously, and sync client is a thin wrapper around async client.

Server is needed to be rewritten to be asynchronous.

Streaming, proper error handling and many many more features are also to be done.
