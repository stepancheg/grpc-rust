#!/bin/sh -ex

cd $(dirname $0)

kill_server() {
    killall -KILL go-grpc-interop-server || true
}

kill_server
./go-grpc-interop-server &

../cargo.sh run --bin grpc-rust-interop-client -- --test_case empty_unary
../cargo.sh run --bin grpc-rust-interop-client -- --test_case large_unary
../cargo.sh run --bin grpc-rust-interop-client -- --test_case ping_pong
../cargo.sh run --bin grpc-rust-interop-client -- --test_case empty_stream

kill_server

set +x

echo "ALL TESTS PASSED"

# vim: set ts=4 sw=4 et:
