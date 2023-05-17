#!/bin/bash

# linux/386
# linux/amd64
# linux/arm/v6
# linux/arm/v7
# linux/arm64/v8
# linux/ppc64le
# linux/s390x
platform="linux"
for arch in arm64 amd64 arm/v6 arm/v7; do 
    for version in 3.17 3.16 3.15 3.14; do 
        docker build . -f Dockerfile.cli.alpine \
            --build-arg=ALPINE_VERSION=$version \
            -t pact-alpine-$platform-$arch-$version \
            --platform=linux/$arch
        docker run --rm -it pact-alpine-$platform-$arch-$version --version
        docker run --rm -it pact-alpine-$platform-$arch-$version --help
    done
done