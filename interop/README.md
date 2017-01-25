grpc-rust interop client/server
===============================

In order to test how well grpc-rust interoperates with other grpc stacks, we
implement the [https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md](standard interop service).

## Current status

Most of the basic interop tests work, but many need improvements.


## Test implementation status

These test names are standard and defined in the above document.

- [x] `empty_unary` empty (zero bytes) request and response.
- [ ] `large_unary` single request and (large) response. #39
- [x] `client_streaming` request streaming with single response.
- [ ] `server_streaming` single request with response streaming. #41
- [x] `ping_pong` full-duplex streaming.
- [x] `empty_stream` full-duplex streaming with zero message.
- [x] `timeout_on_sleeping_server` fullduplex streaming on a sleeping server.
- [ ] `compute_engine_creds` large_unary with compute engine auth. (needs TLS)
- [ ] `service_account_creds` large_unary with service account auth. (needs TLS)
- [ ] `jwt_token_creds` large_unary with jwt token auth. (needs TLS)
- [ ] `per_rpc_creds` large_unary with per rpc token. (needs TLS)
- [ ] `oauth2_auth_token` large_unary with oauth2 token auth. (needs TLS)
- [x] `cancel_after_begin` cancellation after metadata has been sent but before payloads are sent.
- [x] `cancel_after_first_response` cancellation after receiving 1st message from the server.
- [ ] `status_code_and_message` status code propagated back to client. #42
- [ ] `custom_metadata` server will echo custom metadata. #43
- [ ] `unimplemented_method` client attempts to call unimplemented method. #44
- [ ] `unimplemented_service` client attempts to call unimplemented service. (default "large_unary") #45

## How to test a grpc-rust server against the official grpc-go interop client.
# build and run the interop server (from grpc-rust/interop).
```
$ cargo build
$ ../target/debug/grp-interop
```

# build and run the grpc-go interop client.
```
$ go get -u google.golang.org/grpc
$ go get -u cloud.google.com/go/compute/metadata
$ go get -u golang.org/x/oauth2
$ go get -u go/src/github.com/grpc/grpc-go/interop/client/client.go
$ go build -o go-grpc-interop-client  go/src/github.com/grpc/grpc-go/interop/client/client.go
$ ./go-grpc-interop-client -use_tls=false  -test_case=empty_unary -server_port=60011
```

# To find all the test cases you can run, use the --help flag.
`$ ./go-grpc-interop-client --help`

## build and run the grpc-java interop client.
First, you will need gradle installed. (`brew install gradle` on macOS)
```
$ git clone https://github.com/grpc/grpc-java.git
$ cd grpc-java/interop-testing
$ ./gradlew installDist -PskipCodegen=true
$  ./run-test-client.sh --use_tls=false --test_case=empty_unary --server_port=60011
```