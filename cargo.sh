#!/bin/sh -e

if test -z "$TRAVIS" && test -n "$TMPDIR" && rustc -Zhelp | egrep  '\<incremental=' > /dev/null; then
    #export RUSTFLAGS="$RUSTFLAGS -Zincremental=$TMPDIR/grpc-rust"
fi

exec cargo "$@"

# vim: set ts=4 sw=4 et:
