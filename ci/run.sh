#!/bin/sh -e

set -ex

rustc --version

export RUST_BACKTRACE=1

ci/install-protobuf.sh

if test "$ACTION" = "test-protoc-plugin"; then
    (
        cargo install protobuf-codegen

        cd grpc-compiler/test-protoc-plugin

        ./gen.sh

        cargo check --all
    )
else
    cargo test --all --all-targets
fi

# vim: set ts=4 sw=4 et:
