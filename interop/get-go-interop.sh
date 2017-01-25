#!/bin/sh -ex

#go get -u google.golang.org/grpc
#go get -u cloud.google.com/go/compute/metadata
#go get -u golang.org/x/oauth2
go get -u github.com/grpc/grpc-go/interop
go build -o go-grpc-interop-client github.com/grpc/grpc-go/interop/client
go build -o go-grpc-interop-server github.com/grpc/grpc-go/interop/server

# vim: set ts=4 sw=4 et:
