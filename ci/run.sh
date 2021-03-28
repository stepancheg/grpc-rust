#!/bin/sh -e

set -ex

rustc --version

export RUST_BACKTRACE=1

cargo test --all --all-targets

# `--all-targets` does not include doctests
# https://github.com/rust-lang/cargo/issues/6669
cargo test --doc

# Check the docs
cargo doc

# vim: set ts=4 sw=4 et:
