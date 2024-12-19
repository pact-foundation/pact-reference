#!/bin/bash -e

cargo clean
cargo build --release
mkdir -p ../release_artifacts
gzip -c ../target/release/libpact_ffi.dylib > ../release_artifacts/libpact_ffi-osx-x86_64.dylib.gz
openssl dgst -sha256 -r ../release_artifacts/libpact_ffi-osx-x86_64.dylib.gz > ../release_artifacts/libpact_ffi-osx-x86_64.dylib.gz.sha256
gzip -c ../target/release/libpact_ffi.a > ../release_artifacts/libpact_ffi-osx-x86_64.a.gz
openssl dgst -sha256 -r ../release_artifacts/libpact_ffi-osx-x86_64.a.gz > ../release_artifacts/libpact_ffi-osx-x86_64.a.gz.sha256

# M1
export SDKROOT=$(xcrun -sdk macosx11.1 --show-sdk-path)
export MACOSX_DEPLOYMENT_TARGET=$(xcrun -sdk macosx11.1 --show-sdk-platform-version)
cargo build --target aarch64-apple-darwin --release

gzip -c ../target/aarch64-apple-darwin/release/libpact_ffi.dylib > ../release_artifacts/libpact_ffi-osx-aarch64-apple-darwin.dylib.gz
openssl dgst -sha256 -r ../release_artifacts/libpact_ffi-osx-aarch64-apple-darwin.dylib.gz > ../release_artifacts/libpact_ffi-osx-aarch64-apple-darwin.dylib.gz.sha256
gzip -c ../target/aarch64-apple-darwin/release/libpact_ffi.a > ../release_artifacts/libpact_ffi-osx-aarch64-apple-darwin.a.gz
openssl dgst -sha256 -r ../release_artifacts/libpact_ffi-osx-aarch64-apple-darwin.a.gz > ../release_artifacts/libpact_ffi-osx-aarch64-apple-darwin.a.gz.sha256
