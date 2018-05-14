#!/bin/sh -e

cd $(dirname $0)

cargo test --all

# vim: set ts=4 sw=4 et:
