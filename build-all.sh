#!/bin/sh -ex

./cargo.sh build
./cargo.sh build --manifest-path=grpc-compiler/Cargo.toml
./cargo.sh build --manifest-path=grpc-examples/Cargo.toml
./cargo.sh build --manifest-path=long-tests/with-rust/Cargo.toml
./cargo.sh build --manifest-path=interop/Cargo.toml

# vim: set ts=4 sw=4 et:
