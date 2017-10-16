#!/bin/bash
#Copyright: Amdocs Development Limited, 2017
set -e
cd `dirname $0`

#Initialize parameters

GID=$(id -g)

#Build the release in alphine enviornmnet
docker run --rm -e HTTPS_PROXY -v "$(pwd)":/rust -u rust:rust --group-add $GID --group-add sudo -w /rust ekidd/rust-musl-builder \
       bash -c "cargo build --release && sudo chown --reference . -R $uidgid */target"

basedir=$(dirname $0)

#Run over executables
for cli in target/x86_64-unknown-linux-musl/release/*
do
  if [[ -f $cli && -x $cli ]]
  then
    #Create the image
    cliName=$(basename $cli)
    version=$(sed -rn '/\[package\]/,/\[dependencies\]/{s/^version = "([^"]+)"/\1/p}' $cliName/Cargo.toml)
    sed "s/CLI_NAME/$cliName/g" Dockerfile.tpl > target/Dockerfile.cliName
    docker build -f target/Dockerfile.cliName -t $cliName:$version .

    #Push to repostiory
    export DOCKER_ID_USER="assafkatz3"
    docker tag $cliName:$version docker.io/$DOCKER_ID_USER/$cliName:$version
    docker push docker.io/$DOCKER_ID_USER/$cliName:$version
    docker tag $cliName:$version docker.io/$DOCKER_ID_USER/$cliName
    docker push docker.io/$DOCKER_ID_USER/$cliName
  fi
done



