#!/bin/bash -xe

cargo clean

mkdir -p ../release_artifacts
cargo build --release
gzip -c ../target/release/pact_mock_server_cli > ../release_artifacts/pact_mock_server_cli-osx-x86_64.gz
#cargo build --release --target x86_64-apple-ios
#gzip -c ../target/x86_64-apple-ios/release/pact_mock_server_cli > ../release_artifacts/pact_mock_server_cli-ios-x86_64.gz
openssl dgst -sha256 -r ../release_artifacts/pact_mock_server_cli-osx-x86_64.gz > ../release_artifacts/pact_mock_server_cli-osx-x86_64.gz.sha256


# M1
export SDKROOT=$(xcrun -sdk macosx11.1 --show-sdk-path)
export MACOSX_DEPLOYMENT_TARGET=$(xcrun -sdk macosx11.1 --show-sdk-platform-version)
cargo build --target aarch64-apple-darwin --release

gzip -c ../target/aarch64-apple-darwin/release/pact_mock_server_cli > ../release_artifacts/pact_mock_server_cli-osx-aarch64.gz
openssl dgst -sha256 -r ../release_artifacts/pact_mock_server_cli-osx-aarch64.gz > ../release_artifacts/pact_mock_server_cli-osx-aarch64.gz.sha256