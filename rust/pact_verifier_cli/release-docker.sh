#!/bin/bash
set -xe
cd `dirname $0`
dockerenv=""
if [ -n "$HTTPS_PROXY" ]
then
   dockerenv+="-e HTTPS_PROXY=$HTTPS_PROXY"
fi
docker run --rm $dockerenv -it -v "$(pwd)":/rust ekidd/rust-musl-builder bash -xc 'sudo -i cp -R /rust/* `pwd`; sudo -i chown -R rust:rust `pwd`; cargo build --release; sudo -i cp -R `pwd`/target/x86_64-unknown-linux-musl /rust/target; sudo -i chown -R $(stat -c "%u:%g" /rust/target) /rust/target/x86_64-unknown-linux-musl'
docker build -t $(basename $(pwd)) .

