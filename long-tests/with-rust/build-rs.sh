#!/bin/sh -e

cd $(dirname $0)

exec ../../cargo.sh build

# vim: set ts=4 sw=4 et:
