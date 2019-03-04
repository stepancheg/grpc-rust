#!/bin/sh -ex

cd $(dirname $0)

kill_server() {
    killall -KILL go-grpc-interop-server || true
}

kill_server

cargo build --bin grpc-rust-interop-client

./go-grpc-interop-server &

../target/debug/grpc-rust-interop-client --test_case empty_unary
../target/debug/grpc-rust-interop-client --test_case large_unary
../target/debug/grpc-rust-interop-client --test_case ping_pong
../target/debug/grpc-rust-interop-client --test_case empty_stream
../target/debug/grpc-rust-interop-client --test_case custom_metadata

kill_server

set +x

echo "ALL TESTS PASSED"

# vim: set ts=4 sw=4 et:
