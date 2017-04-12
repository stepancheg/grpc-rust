#!/bin/sh -e

cd $(dirname $0)

export RUST_BACKTRACE=1

./cargo.sh test --manifest-path ./http2/Cargo.toml
./cargo.sh test --manifest-path ./grpc-compiler/Cargo.toml
./cargo.sh test

# vim: set ts=4 sw=4 et:
