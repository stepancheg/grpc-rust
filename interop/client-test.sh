#!/bin/sh -ex

cd $(dirname $0)

kill_server() {
    killall -KILL go-grpc-interop-server || true
}

kill_server
./go-grpc-interop-server &

cargo run --bin grpc-rust-interop-client -- --test_case empty_unary
cargo run --bin grpc-rust-interop-client -- --test_case large_unary
cargo run --bin grpc-rust-interop-client -- --test_case ping_pong
cargo run --bin grpc-rust-interop-client -- --test_case empty_stream
cargo run --bin grpc-rust-interop-client -- --test_case custom_metadata

kill_server

set +x

echo "ALL TESTS PASSED"

# vim: set ts=4 sw=4 et:
