#!/bin/sh -e

cd $(dirname $0)

killall -KILL long_tests_server 2> /dev/null || true
killall -KILL long_tests_client 2> /dev/null || true

with-go/long_tests_server/long_tests_server &

go_server_pid=$!

export RUST_BACKTRACE=1

sleep 0.1 # give server some time to start

../target/debug/long_tests_client echo 10000

kill $go_server_pid
