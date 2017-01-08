#!/bin/sh -e

cd $(dirname $0)

export RUSTFLAGS="-Zincremental=$HOME/tmp/grpc-rust"

exec cargo build

# vim: set ts=4 sw=4 et:
