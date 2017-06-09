#!/bin/bash

cd $(dirname $0)

tests=(
    empty_unary
    large_unary
    ping_pong
    empty_stream
    custom_metadata
    status_code_and_message
    unimplemented_method
)

for testname in "${tests[@]}"; do
    ./go-grpc-interop-client -use_tls=false -test_case=$testname
    if [[ $? -ne 0 ]]; then
        exit -1
    fi
done

set +x

echo "ALL TESTS PASSED"

# vim: set ts=4 sw=4 et:
