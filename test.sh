#!/bin/sh -e

cd $(dirname $0)

export RUST_BACKTRACE=1

./cargo.sh test --all

# vim: set ts=4 sw=4 et:
