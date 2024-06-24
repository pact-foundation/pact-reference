#!/bin/bash

set -ex

# Without this fails to compile on Windows
export AWS_LC_SYS_NO_ASM=1

rustc --print cfg
cargo build
cargo test
