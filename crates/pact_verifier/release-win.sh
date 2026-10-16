#!/bin/bash

if [ "" = "$1" ]; then
  echo "Usage: "
  echo "  ./release-win.sh version"
  exit 1
fi

cargo clean
cargo build --release
gzip -c target/release/pact_verifier.dll > target/release/pact_verifier-windows-x86_64-$1.dll.gz
openssl dgst -sha256 -r target/release/pact_verifier-windows-x86_64-$1.dll.gz > target/release/pact_verifier-windows-x86_64-$1.dll.gz.sha256
gzip -c target/release/pact_verifier.lib > target/release/pact_verifier-windows-x86_64-$1.lib.gz
openssl dgst -sha256 -r target/release/pact_verifier-windows-x86_64-$1.lib.gz > target/release/pact_verifier-windows-x86_64-$1.lib.gz.sha256