#!/bin/sh

die() {
    echo "$@" >&2
    exit 1
}

set -ex

cd $(dirname $0)

# for protoc
PATH=$HOME/devel/left/protobuf/src:$GOPATH/bin:$PATH

protoc_ver=$(protoc --version)
case "$protoc_ver" in
    "libprotoc 3"*) ;;
    *) die "protoc version 3 required: $protoc_ver" ;;
esac

protoc -I.. --go_out=plugins=grpc:long_tests_pb ../long_tests_pb.proto

# vim: set ts=4 sw=4 et:
