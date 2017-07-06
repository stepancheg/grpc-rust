#!/bin/sh -e

version="0.2.0"

sed -e 's,^version = .*,version = "'$version'",' -i '' \
    Cargo.toml grpc-compiler/Cargo.toml protoc-rust-grpc/Cargo.toml

sed -e '/grpc/ s,^version = .*,version = "'$version'",' -i '' \
    protoc-rust-grpc/Cargo.toml

# vim: set ts=4 sw=4 et:
