#!/bin/bash

set -ex

rustc --print cfg
cargo build
cargo test
