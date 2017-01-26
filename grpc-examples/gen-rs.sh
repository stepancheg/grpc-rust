#!/bin/sh

set -ex

cd $(dirname $0)

../protoc.sh --rust_out=src helloworld.proto route_guide.proto
../protoc.sh --rust-grpc_out=src helloworld.proto route_guide.proto

# vim: set ts=4 sw=4 et:
