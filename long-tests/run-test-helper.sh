#!/bin/sh -e

die() {
    echo "$@" >&2
    exit 1
}

me=$0

usage() {
    die "usage: $me <rust|go> <rust|go>"
}

cd $(dirname $0)

case $1 in
    rust) client=../target/debug/long_tests_client ;;
    go) client=with-go/long_tests_client/long_tests_client ;;
    *) usage ;;
esac

case $2 in
    rust) server=../target/debug/long_tests_server ;;
    go) server=with-go/long_tests_server/long_tests_server ;;
    *) usage ;;
esac

killall -KILL long_tests_server 2> /dev/null || true
killall -KILL long_tests_client 2> /dev/null || true

$server &

go_server_pid=$!

sleep 0.1 # give server some time to start

$client echo 10000

kill $go_server_pid
