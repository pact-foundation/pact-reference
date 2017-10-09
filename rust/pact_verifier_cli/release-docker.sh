#!/bin/bash
#Copyright: Amdocs Development Limited, 2017
set -e
cd `dirname $0`
dockerenv=""
if [ -n "$HTTPS_PROXY" ]
then
   dockerenv+="-e HTTPS_PROXY=$HTTPS_PROXY"
fi
currdir=$(basename $(pwd))
uidgid=$(stat -c "%u:%g" .)
GID=$(id -g)
docker run --rm $dockerenv -it -v "$(pwd)/..":/rust -u rust:rust --group-add $GID --group-add sudo -w /rust/$currdir ekidd/rust-musl-builder bash -c "cargo build --release && sudo chown -R $uidgid target"
docker build -t $currdir .

