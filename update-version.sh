#!/bin/sh -e

version="0.1.4"

sed -e 's,^version = .*,version = "'$version'",' -i '' \
    Cargo.toml grpc-compiler/Cargo.toml

sed -e '/httpbis.*path/ s,version = [^ ]*,version = "'$version'",' -i '' Cargo.toml

# vim: set ts=4 sw=4 et:
