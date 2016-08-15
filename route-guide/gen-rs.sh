#!/bin/sh

set -ex

cd $(dirname $0)

(
    cd ..
    cargo build
)
(
    cd ../../rust-protobuf
    cargo build
)

PATH=../target/debug:$PATH
PATH=../../rust-protobuf/target/debug:$PATH

$HOME/devel/left/protobuf/src/protoc --rust_out=src route_guide.proto
$HOME/devel/left/protobuf/src/protoc --rust-grpc_out=src route_guide.proto

# vim: set ts=4 sw=4 et:
