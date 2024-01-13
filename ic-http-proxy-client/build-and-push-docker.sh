#!/bin/bash

# build docker image
docker build -t ic-http-proxy-client:latest .

# tag for omniadevs registry
docker tag ic-http-proxy-client:latest omniadevs/ic-http-proxy-client:latest

PACKAGE_VERSION="v$(cat  package.json | grep \"version\" | cut -d'"' -f 4)"

docker image tag ic-http-proxy-client:latest omniadevs/ic-http-proxy-client:$PACKAGE_VERSION
docker image tag ic-http-proxy-client:latest omniadevs/ic-http-proxy-client:latest

# push the image to the registry
docker image push omniadevs/ic-http-proxy-client:$PACKAGE_VERSION
docker image push omniadevs/ic-http-proxy-client:latest
