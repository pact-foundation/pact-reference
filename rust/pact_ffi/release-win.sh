#!/bin/bash -e

cargo clean
cargo build --release
mkdir -p ../release_artifacts

gzip -c ../target/release/pact_ffi.dll > ../release_artifacts/pact_ffi-windows-x86_64.dll.gz
openssl dgst -sha256 -r ../release_artifacts/pact_ffi-windows-x86_64.dll.gz > ../release_artifacts/pact_ffi-windows-x86_64.dll.gz.sha256
gzip -c ../target/release/pact_ffi.dll.lib > ../release_artifacts/pact_ffi-windows-x86_64.dll.lib.gz
openssl dgst -sha256 -r ../release_artifacts/pact_ffi-windows-x86_64.dll.lib.gz > ../release_artifacts/pact_ffi-windows-x86_64.dll.lib.gz.sha256
gzip -c ../target/release/pact_ffi.lib > ../release_artifacts/pact_ffi-windows-x86_64.lib.gz
openssl dgst -sha256 -r ../release_artifacts/pact_ffi-windows-x86_64.lib.gz > ../release_artifacts/pact_ffi-windows-x86_64.lib.gz.sha256

echo -- Build the aarch64 release artifacts --
cargo build --target aarch64-pc-windows-msvc --release
gzip -c ../target/aarch64-pc-windows-msvc/release/pact_ffi.dll > ../release_artifacts/pact_ffi-windows-aarch64.dll.gz
openssl dgst -sha256 -r ../release_artifacts/pact_ffi-windows-aarch64.dll.gz > ../release_artifacts/pact_ffi-windows-aarch64.dll.gz.sha256
gzip -c ../target/aarch64-pc-windows-msvc/release/pact_ffi.dll.lib > ../release_artifacts/pact_ffi-windows-aarch64.dll.lib.gz
openssl dgst -sha256 -r ../release_artifacts/pact_ffi-windows-aarch64.dll.lib.gz > ../release_artifacts/pact_ffi-windows-aarch64.dll.lib.gz.sha256
gzip -c ../target/aarch64-pc-windows-msvc/release/pact_ffi.lib > ../release_artifacts/pact_ffi-windows-aarch64.lib.gz
openssl dgst -sha256 -r ../release_artifacts/pact_ffi-windows-aarch64.lib.gz > ../release_artifacts/pact_ffi-windows-aarch64.lib.gz.sha256
