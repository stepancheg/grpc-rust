#!/bin/sh -e

set -ex

rustc --version

export RUST_BACKTRACE=1

ci/install-protobuf.sh

cargo test --all --all-targets

(
    cargo install protobuf-codegen --version '2'

    cd grpc-compiler/test-protoc-plugin

    ./gen.sh

    cargo check --all
)

# vim: set ts=4 sw=4 et:
