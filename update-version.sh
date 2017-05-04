#!/bin/sh -e

sed -e 's,^version = .*,version = "0.1.2",' -i '' \
    Cargo.toml http2/Cargo.toml grpc-compiler/Cargo.toml

sed -e '/httpbis.*path/ s,version = [^ ]*,version = "0.1.2",' -i '' Cargo.toml

# vim: set ts=4 sw=4 et:
