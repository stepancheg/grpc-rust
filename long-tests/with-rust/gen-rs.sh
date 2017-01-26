#!/bin/sh

set -ex

cd $(dirname $0)

../../protoc.sh -I.. --rust_out=src ../long_tests_pb.proto
../../protoc.sh -I.. --rust-grpc_out=src ../long_tests_pb.proto

# vim: set ts=4 sw=4 et:
