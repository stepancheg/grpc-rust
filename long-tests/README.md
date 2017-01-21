## Compatibility tests

This directory contains tests against Go implementation of gRPC.

Basically it contains the same implementation in Rust and in Go:
Go or Rust client should behave identically when connected to the server
implemented in Go or Rust.

## How to use it

```
# Rust client and Go server
% ./run-test-helper.sh rust go
running 10000 iterations of echo
done
# Go client and Rust server
% ./run-test-helper.sh go rust
2017/01/21 23:38:49 running 10000 iterations of echo
2017/01/21 23:38:51 done
```
