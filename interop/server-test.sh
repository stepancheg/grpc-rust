#!/bin/bash -ex

cd $(dirname $0)

kill_server() {
    killall -KILL grpc-rust-interop-server || true
}

cargo build
kill_server
../target/debug/grpc-rust-interop-server &

trap kill_server EXIT

tests=(
    empty_unary
    large_unary
    ping_pong
    empty_stream
    custom_metadata
    status_code_and_message
    unimplemented_method
    cancel_after_first_response
)

for testname in "${tests[@]}"; do
    ./go-grpc-interop-client -use_tls=false -test_case=$testname
done

kill_server

set +x

echo "ALL TESTS PASSED"

# vim: set ts=4 sw=4 et:
