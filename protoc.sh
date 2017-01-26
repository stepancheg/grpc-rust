#!/bin/sh -e

(
    cd $(dirname $0)

    cd grpc-compiler
    ../cargo.sh build
)

# for protoc-gen-rust-grpc
PATH=$(dirname $0)/target/debug:$PATH
# for protoc-gen-rust
#PATH=../../rust-protobuf/target/debug:$PATH
# for protoc
PATH=$HOME/devel/left/protobuf/src:$PATH

protoc_ver=$(protoc --version)
case "$protoc_ver" in
    "libprotoc 3"*) ;;
    *) die "protoc version 3 required: $protoc_ver" ;;
esac

exec protoc "$@"

# vim: set ts=4 sw=4 et:
