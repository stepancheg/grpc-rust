#!/bin/sh

set -ex

cd $(dirname $0)

../protoc.sh -Iproto --rust_out=src proto/messages.proto
../protoc.sh -Iproto --rust_out=src proto/empty.proto
../protoc.sh -Iproto --rust_out=src proto/test.proto

../protoc.sh -Iproto --rust-grpc_out=src proto/messages.proto
../protoc.sh -Iproto --rust-grpc_out=src proto/empty.proto
../protoc.sh -Iproto --rust-grpc_out=src proto/test.proto

# vim: set ts=4 sw=4 et:
