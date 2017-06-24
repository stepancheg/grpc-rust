#!/bin/sh -e

version="0.1.10"

sed -e 's,^version = .*,version = "'$version'",' -i '' \
    Cargo.toml grpc-compiler/Cargo.toml protoc-rust-grpc/Cargo.toml

# vim: set ts=4 sw=4 et:
