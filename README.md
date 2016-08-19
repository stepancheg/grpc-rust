# grpc-rust

Rust implementation of [gRPC](http://www.grpc.io/) protocol, under development.

## Current status

Synchronous server without streaming can be done with rust-grpc,
see `grpc-examples/src/bin/greeter_client.rs`. It can be tested
for example with go client:

```
# start greeter server implemented in rust
$ cargo run --bin greeter_server

# get go greeter client
$ go get -u google.golang.org/grpc/examples/helloworld/greeter_client

# call the server
$ greeter_client rust
2016/08/19 05:44:45 Greeting: Hello rust
```

Client, asynchronous client, asynchronous server, streaming are to be implemented.
