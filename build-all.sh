#!/bin/sh -ex

cargo build
cargo build --manifest-path=grpc-compiler/Cargo.toml
cargo build --manifest-path=grpc-examples/Cargo.toml
cargo build --manifest-path=long-tests/with-rust/Cargo.toml
cargo build --manifest-path=interop/Cargo.toml

# vim: set ts=4 sw=4 et:
