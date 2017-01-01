#!/bin/sh -e

cd $(dirname $0)

export RUST_BACKTRACE=1

cargo test --manifest-path ./http2/Cargo.toml
cargo test

# vim: set ts=4 sw=4 et:
