#!/bin/sh -e

version="0.3.1"

sed -e 's,^version = .*,version = "'$version'",' -i '' \
    grpc/Cargo.toml grpc-compiler/Cargo.toml protoc-rust-grpc/Cargo.toml

sed -e '/grpc/ s,version = "[^"]*",version = "='$version'",' -i '' \
    protoc-rust-grpc/Cargo.toml

# vim: set ts=4 sw=4 et:
