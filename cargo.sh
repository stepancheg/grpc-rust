#!/bin/sh -e

if rustc rustc -Zhelp | egrep  '\<incremental=' > /dev/null; then
    RUSTFLAGS="$RUSTFLAGS -Zincremental=$TMPDIR/grpc-rust"
fi

exec cargo "$@"

# vim: set ts=4 sw=4 et:
