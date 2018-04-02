# Test against go client

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

# Test TLS

Rust client and server can talk via TLS.

```
$ cargo run --bin greeter_server -- --tls

$ cargo run --bin greeter_client -- --tls
```
