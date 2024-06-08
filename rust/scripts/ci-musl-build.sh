#!/bin/bash

set -ex

rustc --print cfg
cargo build
cargo test
cargo test -p pact_ffi returns_mock_server_logs -- --nocapture --include-ignored
