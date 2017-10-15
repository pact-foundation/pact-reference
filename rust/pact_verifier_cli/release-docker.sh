#!/bin/bash
#Copyright: Amdocs Development Limited, 2017
set -e
cd `dirname $0`

#Handle proxy
dockerenv=""
if [ -n "$HTTPS_PROXY" ]
then
   dockerenv+="-e HTTPS_PROXY=$HTTPS_PROXY"
fi

#Initialize parameters
currdir=$(basename $(pwd))
uidgid=$(stat -c "%u:%g" .)
GID=$(id -g)
version=$(sed -rn '/\[package\]/,/\[dependencies\]/{s/^version = "([^"]+)"/\1/p}' Cargo.toml)

#Build the release in alphine enviornmnet
docker run --rm $dockerenv -it -v "$(pwd)/..":/rust -u rust:rust --group-add $GID --group-add sudo -w /rust/$currdir ekidd/rust-musl-builder \
       bash -c "cargo build --release && sudo chown -R $uidgid target"

#Create the image
docker build -t $currdir:$version .

#Push to repostiory
#export DOCKER_ID_USER="assafkatz3"
#docker tag $currdir:$version docker.io/$DOCKER_ID_USER/$currdir:$version
#docker push docker.io/$DOCKER_ID_USER/$currdir:$version
#docker tag $currdir:$version docker.io/$DOCKER_ID_USER/$currdir
#docker push docker.io/$DOCKER_ID_USER/$currdir

