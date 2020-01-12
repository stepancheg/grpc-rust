#!/bin/sh -e

set -ex

rustc --versoin

export RUST_BACKTRACE=1

ci/install-protobuf.sh

cargo install protobuf-codegen --version '2'

cargo test --all

./grpc-compiler/test-protoc-plugin/gen.sh

cargo check --all

# vim: set ts=4 sw=4 et:
