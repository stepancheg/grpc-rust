#!/bin/sh -e

cd $(dirname $0)

export RUSTFLAGS="-Zincremental=$HOME/tmp/grpc-rust"

exec cargo build --manifest-path ./with-rust/Cargo.toml

# vim: set ts=4 sw=4 et:
