grpc-rust interop client/server
===============================

In order to test how well grpc-rust interoperates with other grpc stacks, we
implement the [https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md](standard interop service).

## Current status

Most of the basic interop tests work, but many need improvements.


## Test implementation status

Covered in issue #47

## How to test a grpc-rust server against the official grpc-go interop client.
# build and run the interop server (from grpc-rust/interop).
```
$ cargo build
$ ../target/debug/grp-interop
```

# build and run the grpc-go interop client
```
$ ./get-go-interop.sh
$ ./go-grpc-interop-client -use_tls=false  -test_case=empty_unary -server_port=60011
```

# To find all the test cases you can run, use the --help flag.
`$ ./go-grpc-interop-client --help`

## build and run the grpc-java interop client.
First, you will need gradle installed. (`brew install gradle` on macOS)
```
$ git clone https://github.com/grpc/grpc-java.git
$ cd grpc-java
$ ./gradlew installDist -PskipCodegen=true
$  ./run-test-client.sh --use_tls=false --test_case=empty_unary --server_port=60011
```
