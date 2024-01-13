#!/bin/bash

# build docker image
docker build -t ic-http-request-executor:latest .

# tag for omniadevs registry
docker tag ic-http-request-executor:latest omniadevs/ic-http-request-executor:latest

PACKAGE_VERSION="v$(cat  package.json | grep \"version\" | cut -d'"' -f 4)"

docker image tag ic-http-request-executor:latest omniadevs/ic-http-request-executor:$PACKAGE_VERSION
docker image tag ic-http-request-executor:latest omniadevs/ic-http-request-executor:latest

# push the image to the registry
docker image push omniadevs/ic-http-request-executor:$PACKAGE_VERSION
docker image push omniadevs/ic-http-request-executor:latest
