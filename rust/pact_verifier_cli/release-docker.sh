#!/bin/bash

set -e

if [ "" = "$1" ]; then
  echo "Usage: "
  echo "  ./release-docker.sh version"
  exit 1
fi

# AMD64
docker build . -t pactfoundation/pact-verifier:$1-amd64 --platform linux/amd64 \
    --build-arg ARCH=amd64/ --build-arg BIN_ARCH=x86_64 --build-arg VERSION=$1
docker push pactfoundation/pact-verifier:$1-amd64

# ARM64V8
docker build . -t pactfoundation/pact-verifier:$1-arm64v8 --platform linux/arm64 \
    --build-arg ARCH=arm64v8/ --build-arg BIN_ARCH=aarch64 --build-arg VERSION=$1
docker push pactfoundation/pact-verifier:$1-arm64v8

# Create Manifest
docker manifest create pactfoundation/pact-verifier:$1 \
    --amend pactfoundation/pact-verifier:$1-amd64 \
    --amend pactfoundation/pact-verifier:$1-arm64v8
docker manifest push pactfoundation/pact-verifier:$1
docker manifest create pactfoundation/pact-verifier:latest \
    --amend pactfoundation/pact-verifier:$1-amd64 \
    --amend pactfoundation/pact-verifier:$1-arm64v8
docker manifest push pactfoundation/pact-verifier:latest
