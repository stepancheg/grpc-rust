#!/bin/sh -ex

./interop/gen-rs.sh
./grpc-examples/gen-rs.sh
./long-tests/with-rust/gen-rs.sh

# vim: set ts=4 sw=4 et:
