#!/bin/sh -e

cd $(dirname $0)

# Generate root CA

openssl genrsa -out root-ca.key 1024

openssl req -x509 -new -nodes -subj '/CN=www.mydom.com/O=My Company Name LTD./C=US' -key root-ca.key -sha256 -days 1024 -out root-ca.crt

openssl x509 -outform der -in root-ca.crt -out root-ca.der


# Generate server certificate

openssl genrsa -out foobar.com.key 1024

openssl req -new -key foobar.com.key -subj '/CN=foobar.com/O=My Company Name LTD./C=US' -out foobar.com.csr

openssl x509 -req -in foobar.com.csr -CA root-ca.crt -CAkey root-ca.key -CAcreateserial -out foobar.com.crt -days 500 -sha256

openssl pkcs12 -export -inkey foobar.com.key -in foobar.com.crt -out foobar.com.p12 -password pass:mypass


# Cleanup

rm foobar.com.crt foobar.com.csr foobar.com.key root-ca.crt root-ca.key root-ca.srl


# vim: set ts=4 sw=4 et:
