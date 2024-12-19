#!/bin/bash -x

set -e

echo -- Setup directories --
cargo clean
mkdir -p ../release_artifacts

echo -- Build the Docker build image --
docker build -f Dockerfile.linux-build -t pact-ffi-build .

echo -- Build the release artifacts --
docker run -t --rm --user "$(id -u)":"$(id -g)" -v $(pwd)/..:/workspace -w /workspace/pact_ffi pact-ffi-build -c 'cargo build --release'
gzip -c ../target/release/libpact_ffi.so > ../release_artifacts/libpact_ffi-linux-x86_64.so.gz
openssl dgst -sha256 -r ../release_artifacts/libpact_ffi-linux-x86_64.so.gz > ../release_artifacts/libpact_ffi-linux-x86_64.so.gz.sha256
gzip -c ../target/release/libpact_ffi.a > ../release_artifacts/libpact_ffi-linux-x86_64.a.gz
openssl dgst -sha256 -r ../release_artifacts/libpact_ffi-linux-x86_64.a.gz > ../release_artifacts/libpact_ffi-linux-x86_64.a.gz.sha256

echo -- Generate the header files --
rustup toolchain install nightly
rustup component add rustfmt --toolchain nightly
rustup run nightly cbindgen \
  --config cbindgen.toml \
  --crate pact_ffi \
  --output include/pact.h
rustup run nightly cbindgen \
  --config cbindgen-c++.toml \
  --crate pact_ffi \
  --output include/pact-cpp.h
cp include/*.h ../release_artifacts

echo -- Build the musl release artifacts --
cargo install cross@0.2.5
cross build --release --target=x86_64-unknown-linux-musl
gzip -c ../target/x86_64-unknown-linux-musl/release/libpact_ffi.a > ../release_artifacts/libpact_ffi-linux-x86_64-musl.a.gz
openssl dgst -sha256 -r ../release_artifacts/libpact_ffi-linux-x86_64-musl.a.gz > ../release_artifacts/libpact_ffi-linux-x86_64-musl.a.gz.sha256

mkdir tmp 
cp ../target/x86_64-unknown-linux-musl/release/libpact_ffi.a tmp/
docker run --platform=linux/amd64 --rm -v $PWD/tmp:/scratch alpine /bin/sh -c 'apk add --no-cache musl-dev gcc && \ 
cd /scratch && \
    ar -x libpact_ffi.a && \
    gcc -shared *.o -o libpact_ffi.so && \
    rm -f *.o'

gzip -c tmp/libpact_ffi.so > ../release_artifacts/libpact_ffi-linux-x86_64-musl.so.gz
openssl dgst -sha256 -r ../release_artifacts/libpact_ffi-linux-x86_64-musl.so.gz > ../release_artifacts/libpact_ffi-linux-x86_64-musl.so.gz.sha256
rm -rf tmp


echo -- Build the musl aarch64 release artifacts --
cargo clean
cross build --release --target=aarch64-unknown-linux-musl
gzip -c ../target/aarch64-unknown-linux-musl/release/libpact_ffi.a > ../release_artifacts/libpact_ffi-linux-aarch64-musl.a.gz
openssl dgst -sha256 -r ../release_artifacts/libpact_ffi-linux-aarch64-musl.a.gz > ../release_artifacts/libpact_ffi-linux-aarch64-musl.a.gz.sha256

mkdir tmp 
cp ../target/aarch64-unknown-linux-musl/release/libpact_ffi.a tmp/
docker run --platform=linux/arm64 --rm -v $PWD/tmp:/scratch alpine /bin/sh -c 'apk add --no-cache musl-dev gcc && \ 
cd /scratch && \
    ar -x libpact_ffi.a && \
    gcc -shared *.o -o libpact_ffi.so && \
    rm -f *.o'

gzip -c tmp/libpact_ffi.so > ../release_artifacts/libpact_ffi-linux-aarch64-musl.so.gz
openssl dgst -sha256 -r ../release_artifacts/libpact_ffi-linux-aarch64-musl.so.gz > ../release_artifacts/libpact_ffi-linux-aarch64-musl.so.gz.sha256
rm -rf tmp

echo -- Build the aarch64 release artifacts --
cargo clean
cross build --target aarch64-unknown-linux-gnu --release
gzip -c ../target/aarch64-unknown-linux-gnu/release/libpact_ffi.so > ../release_artifacts/libpact_ffi-linux-aarch64.so.gz
openssl dgst -sha256 -r ../release_artifacts/libpact_ffi-linux-aarch64.so.gz > ../release_artifacts/libpact_ffi-linux-aarch64.so.gz.sha256
gzip -c ../target/aarch64-unknown-linux-gnu/release/libpact_ffi.a > ../release_artifacts/libpact_ffi-linux-aarch64.a.gz
openssl dgst -sha256 -r ../release_artifacts/libpact_ffi-linux-aarch64.a.gz > ../release_artifacts/libpact_ffi-linux-aarch64.a.gz.sha256
