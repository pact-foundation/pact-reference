#!/bin/bash -x

set -e

mkdir -p ../target/artifacts
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
          cp include/*.h ../target/artifacts
          ls ../target/artifacts