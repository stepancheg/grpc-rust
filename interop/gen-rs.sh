#!/bin/sh

die() {
    echo "$@" >&2
    exit 1
}

set -ex

cd $(dirname $0)

#(
#    cd ../../grpc-compiler
#    ../cargo.sh build
#)
#(
#    cd ../../../rust-protobuf
#    cargo build
#)

# for protoc-gen-rust-grpc
PATH=../target/debug:$PATH
# for protoc-gen-rust
PATH=../rust-protobuf/target/debug:$PATH
# for protoc
PATH=$HOME/devel/left/protobuf/src:$PATH

protoc_ver=$(protoc --version)
case "$protoc_ver" in
    "libprotoc 3"*) ;;
    *) die "protoc version 3 required: $protoc_ver" ;;
esac

protoc -Iproto --rust_out=src proto/messages.proto
protoc -Iproto --rust_out=src proto/empty.proto
protoc -Iproto --rust_out=src proto/test.proto

protoc -Iproto --rust-grpc_out=src proto/messages.proto
protoc -Iproto --rust-grpc_out=src proto/empty.proto
protoc -Iproto --rust-grpc_out=src proto/test.proto

# vim: set ts=4 sw=4 et:
